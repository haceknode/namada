//! IBC validity predicate for port module

use std::str::FromStr;

use ibc::ics04_channel::context::ChannelReader;
use ibc::ics05_port::capabilities::Capability;
use ibc::ics05_port::context::PortReader;
use ibc::ics24_host::identifier::PortId;
use ibc::ics24_host::Path;

use super::{Error, Ibc, Result, StateChange};
use crate::ledger::storage::{self, StorageHasher};
use crate::types::storage::{Key, KeySeg};

impl<'a, DB, H> Ibc<'a, DB, H>
where
    DB: 'static + storage::DB + for<'iter> storage::DBIter<'iter>,
    H: 'static + StorageHasher,
{
    pub(super) fn validate_port(&self, key: &Key) -> Result<bool> {
        let port_id = Self::get_port_id(key)?;
        match self.get_port_state_change(&port_id)? {
            StateChange::Created | StateChange::Updated => {
                match self.authenticated_capability(&port_id) {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        tracing::info!("{}", e);
                        Ok(false)
                    }
                }
            }
            _ => {
                tracing::info!(
                    "unexpected state change of the port: {}",
                    port_id
                );
                Ok(false)
            }
        }
    }

    /// Returns the port ID after #IBC/channelEnds/ports
    pub(super) fn get_port_id(key: &Key) -> Result<PortId> {
        match key.segments.get(3) {
            Some(id) => PortId::from_str(&id.raw())
                .map_err(|e| Error::KeyError(e.to_string())),
            None => Err(Error::KeyError(format!(
                "The key doesn't have a port ID: {}",
                key
            ))),
        }
    }

    fn get_port_state_change(&self, port_id: &PortId) -> Result<StateChange> {
        let path = Path::Ports(port_id.clone()).to_string();
        let key =
            Key::ibc_key(path).expect("Creating a key for a connection failed");
        self.get_state_change(&key)
    }
}

impl<'a, DB, H> PortReader for Ibc<'a, DB, H>
where
    DB: 'static + storage::DB + for<'iter> storage::DBIter<'iter>,
    H: 'static + StorageHasher,
{
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<Capability> {
        let path = Path::Ports(port_id.clone()).to_string();
        let key = Key::ibc_key(path).expect("Creating a key for a port failed");
        match self.ctx.read_post(&key) {
            // TODO fix Capability in `ibc-rs` to set the index
            Ok(Some(_)) => Some(Capability::new()),
            _ => None,
        }
    }

    fn authenticate(&self, _cap: &Capability, _port_id: &PortId) -> bool {
        // TODO check the reversed map for the capability index
        true
    }
}
