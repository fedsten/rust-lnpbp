// LNP/BP Core Library implementing LNPBP specifications & standards
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::Bipolar;
use std::borrow::Borrow;

use lightning::ln::peers::encryption::{Decryptor, Encryptor};
use lightning::ln::peers::handshake::CompletedHandshakeInfo;
//use lightning::ln::peers::handshake::PeerHandshake;

pub struct Transcoder(CompletedHandshakeInfo);

pub trait Encrypt {
    fn encrypt(&mut self, buffer: impl Borrow<[u8]>) -> Vec<u8>;
}

pub trait Decrypt {
    type Error: ::std::error::Error;

    fn decrypt(
        &mut self,
        buffer: impl Borrow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error>;
}

pub trait Transcode: Bipolar + Encrypt + Decrypt {
    type Encryptor: Encrypt;
    type Decryptor: Decrypt;
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Error,
)]
#[display(Debug)]
pub struct DecryptionError;

impl Encrypt for Encryptor {
    fn encrypt(&mut self, buffer: impl Borrow<[u8]>) -> Vec<u8> {
        self.encrypt_buf(buffer.borrow())
    }
}

impl Decrypt for Decryptor {
    type Error = DecryptionError;

    fn decrypt(
        &mut self,
        buffer: impl Borrow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        match self.decrypt_next(buffer.borrow()) {
            Ok((Some(data), _)) => Ok(data),
            _ => Err(DecryptionError),
        }
    }
}

impl Encrypt for Transcoder {
    fn encrypt(&mut self, buffer: impl Borrow<[u8]>) -> Vec<u8> {
        self.0.encryptor.encrypt_buf(buffer.borrow())
    }
}

impl Decrypt for Transcoder {
    type Error = DecryptionError;

    fn decrypt(
        &mut self,
        buffer: impl Borrow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        match self.0.decryptor.decrypt_next(buffer.borrow()) {
            Ok((Some(data), _)) => Ok(data),
            _ => Err(DecryptionError),
        }
    }
}

impl Transcode for Transcoder {
    type Encryptor = Encryptor;
    type Decryptor = Decryptor;
}

impl Bipolar for Transcoder {
    type Left = <Self as Transcode>::Encryptor;
    type Right = <Self as Transcode>::Decryptor;

    /// Creates conduit by joining encrypting and decrypting parts
    fn join(encryptor: Self::Left, decryptor: Self::Right) -> Self {
        // TODO: (new) figure out what to do with `their_node_id` field
        Self(CompletedHandshakeInfo {
            decryptor,
            encryptor,
            their_node_id: bitcoin::secp256k1::PublicKey::from_slice(
                &[0u8; 33],
            )
            .unwrap(),
        })
    }

    /// Splits conduit into an encrypting and decrypting parts
    fn split(self) -> (Self::Left, Self::Right) {
        (self.0.encryptor, self.0.decryptor)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Display)]
#[display(Debug)]
pub struct NoEncryption;

impl Encrypt for NoEncryption {
    fn encrypt(&mut self, buffer: impl Borrow<[u8]>) -> Vec<u8> {
        buffer.borrow().to_vec()
    }
}

impl Decrypt for NoEncryption {
    type Error = !;
    fn decrypt(
        &mut self,
        buffer: impl Borrow<[u8]>,
    ) -> Result<Vec<u8>, Self::Error> {
        Ok(buffer.borrow().to_vec())
    }
}

impl Transcode for NoEncryption {
    type Encryptor = Self;
    type Decryptor = Self;
}

impl Bipolar for NoEncryption {
    type Left = <Self as Transcode>::Encryptor;
    type Right = <Self as Transcode>::Decryptor;

    fn join(encryptor: Self::Left, _decryptor: Self::Right) -> Self {
        encryptor as NoEncryption
    }

    fn split(self) -> (Self::Left, Self::Right) {
        (self.clone(), self)
    }
}
