// Copyright 2018-2023 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use anyhow::Result;
use contract_build::Verbosity;
use derivative::Derivative;
use ink_env::Environment;
use subxt::{
    tx,
    Config,
};
use url::Url;

use crate::{
    prod_chains::ProductionChain,
    url_to_string,
    ContractArtifacts,
};
use std::{
    marker::PhantomData,
    option::Option,
    path::PathBuf,
};

#[derive(Debug)]
pub enum Chain {
    Production(String),
    Custom,
}

/// Arguments required for creating and sending an extrinsic to a substrate node.
#[derive(Derivative)]
#[derivative(Clone(bound = "E::Balance: Clone"))]
pub struct ExtrinsicOpts<C: Config, E: Environment, Signer: Clone> {
    file: Option<PathBuf>,
    manifest_path: Option<PathBuf>,
    url: url::Url,
    signer: Signer,
    storage_deposit_limit: Option<E::Balance>,
    verbosity: Verbosity,
    chain: Option<ProductionChain>,
    _marker: PhantomData<C>,
}

/// A builder for extrinsic options.
pub struct ExtrinsicOptsBuilder<C: Config, E: Environment, Signer: Clone> {
    opts: ExtrinsicOpts<C, E, Signer>,
}

impl<C: Config, E: Environment, Signer> ExtrinsicOptsBuilder<C, E, Signer>
where
    Signer: tx::Signer<C> + Clone,
{
    /// Returns a clean builder for [`ExtrinsicOpts`].
    pub fn new(signer: Signer) -> ExtrinsicOptsBuilder<C, E, Signer> {
        ExtrinsicOptsBuilder {
            opts: ExtrinsicOpts {
                file: None,
                manifest_path: None,
                url: url::Url::parse("ws://localhost:9944").unwrap(),
                signer,
                storage_deposit_limit: None,
                verbosity: Verbosity::Default,
                chain: None,
                _marker: PhantomData,
            },
        }
    }

    /// Sets the path to the contract build artifact file.
    pub fn file<T: Into<PathBuf>>(self, file: Option<T>) -> Self {
        let mut this = self;
        this.opts.file = file.map(|f| f.into());
        this
    }

    /// Sets the path to the Cargo.toml of the contract.
    pub fn manifest_path<T: Into<PathBuf>>(self, manifest_path: Option<T>) -> Self {
        let mut this = self;
        this.opts.manifest_path = manifest_path.map(|f| f.into());
        this
    }

    /// Sets the websockets url of a substrate node.
    pub fn url<T: Into<Url>>(self, url: T) -> Self {
        let mut this = self;
        this.opts.url = url.into();
        this
    }

    /// Sets the maximum amount of balance that can be charged from the caller to pay for
    /// storage.
    pub fn storage_deposit_limit(
        self,
        storage_deposit_limit: Option<E::Balance>,
    ) -> Self {
        let mut this = self;
        this.opts.storage_deposit_limit = storage_deposit_limit;
        this
    }

    /// Set the verbosity level.
    pub fn verbosity(self, verbosity: Verbosity) -> Self {
        let mut this = self;
        this.opts.verbosity = verbosity;
        this
    }

    /// Set the production chain.
    pub fn chain(self, chain: Option<ProductionChain>) -> Self {
        let mut this = self;
        this.opts.chain = chain;
        this
    }

    pub fn done(self) -> ExtrinsicOpts<C, E, Signer> {
        self.opts
    }
}

impl<C: Config, E: Environment, Signer> ExtrinsicOpts<C, E, Signer>
where
    Signer: tx::Signer<C> + Clone,
{
    /// Load contract artifacts.
    pub fn contract_artifacts(&self) -> Result<ContractArtifacts> {
        ContractArtifacts::from_manifest_or_file(
            self.manifest_path.as_ref(),
            self.file.as_ref(),
        )
    }

    /// Return the file path of the contract artifact.
    pub fn file(&self) -> Option<&PathBuf> {
        self.file.as_ref()
    }

    /// Return the path to the `Cargo.toml` of the contract.
    pub fn manifest_path(&self) -> Option<&PathBuf> {
        self.manifest_path.as_ref()
    }

    /// Return the URL of the substrate node.
    pub fn url(&self) -> String {
        url_to_string(&self.url)
    }

    /// Get the chain name and its URL endpoint.
    /// If the user specify the endpoint manually,
    /// but it still appears to be the production chain,
    /// we still convert it.
    pub fn chain_and_endpoint(&self) -> (Chain, String) {
        if let Some(chain) = &self.chain {
            (
                Chain::Production(chain.to_string()),
                chain.end_point().to_string(),
            )
        } else {
            let url = self.url();
            if let Some(chain) = ProductionChain::chain_by_endpoint(&url) {
                (
                    Chain::Production(chain.to_string()),
                    chain.end_point().to_string(),
                )
            } else {
                (Chain::Custom, url)
            }
        }
    }

    pub fn is_verifiable(&self) -> Result<bool> {
        Ok(self.contract_artifacts()?.is_verifiable())
    }

    /// Return the signer.
    pub fn signer(&self) -> &Signer {
        &self.signer
    }

    /// Return the storage deposit limit.
    pub fn storage_deposit_limit(&self) -> Option<E::Balance> {
        self.storage_deposit_limit
    }

    /// Verbosity for message reporting.
    pub fn verbosity(&self) -> &Verbosity {
        &self.verbosity
    }
}
