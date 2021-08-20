//! Types for the IBC events emitted from Tendermint Websocket by the connection module.
use crate::events::{extract_attribute, maybe_extract_attribute, Error, IbcEvent, RawObject};
use prusti_contracts::*;
use crate::ics02_client::height::Height;
use crate::ics24_host::identifier::{ClientId, ConnectionId};
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;

/// The content of the `type` field for the event that a chain produces upon executing a connection handshake transaction.
const INIT_EVENT_TYPE: &str = "connection_open_init";
const TRY_EVENT_TYPE: &str = "connection_open_try";
const ACK_EVENT_TYPE: &str = "connection_open_ack";
const CONFIRM_EVENT_TYPE: &str = "connection_open_confirm";

/// The content of the `key` field for the attribute containing the connection identifier.
const CONN_ID_ATTRIBUTE_KEY: &str = "connection_id";
const CLIENT_ID_ATTRIBUTE_KEY: &str = "client_id";
const COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY: &str = "counterparty_connection_id";
const COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY: &str = "counterparty_client_id";

pub fn try_from_tx(event: &tendermint::abci::Event) -> Option<IbcEvent> {
    match event.type_str.as_ref() {
        INIT_EVENT_TYPE => Some(IbcEvent::OpenInitConnection(OpenInit::from(
            extract_attributes_from_tx(event),
        ))),
        TRY_EVENT_TYPE => Some(IbcEvent::OpenTryConnection(OpenTry::from(
            extract_attributes_from_tx(event),
        ))),
        ACK_EVENT_TYPE => Some(IbcEvent::OpenAckConnection(OpenAck::from(
            extract_attributes_from_tx(event),
        ))),
        CONFIRM_EVENT_TYPE => Some(IbcEvent::OpenConfirmConnection(OpenConfirm::from(
            extract_attributes_from_tx(event),
        ))),
        _ => None,
    }
}

#[cfg(feature="prusti")]
#[extern_spec]
impl<T, E: std::fmt::Debug> Result<T, E> {
    #[pure]
    pub fn is_ok(&self) -> bool {
        match self {
            Ok(_) => true,
            Err(_) => false
        }
    }
    #[requires(self.is_ok())]
    pub fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => unreachable!()
        }
    }
}

fn extract_attributes_from_tx(event: &tendermint::abci::Event) -> Attributes {
    let mut attr = Attributes::default();

    for tag in &event.attributes {
        let key = tag.key.as_ref();
        let value = tag.value.as_ref();
        match key {
            CONN_ID_ATTRIBUTE_KEY => attr.connection_id = value.parse().ok(),
            CLIENT_ID_ATTRIBUTE_KEY => attr.client_id = value.parse().unwrap(),
            COUNTERPARTY_CONN_ID_ATTRIBUTE_KEY => {
                attr.counterparty_connection_id = value.parse().ok()
            }
            COUNTERPARTY_CLIENT_ID_ATTRIBUTE_KEY => {
                attr.counterparty_client_id = value.parse().unwrap()
            }
            // TODO: `Attributes` has 5 fields and we're only parsing 4
            _ => {}
        }
    }

    attr
}

#[derive(Clone, Hash)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub struct Attributes {
    pub height: Height,
    pub connection_id: Option<ConnectionId>,
    pub client_id: ClientId,
    pub counterparty_connection_id: Option<ConnectionId>,
    pub counterparty_client_id: ClientId,
}

#[cfg(feature="prusti")]
#[trusted]
fn extract_attributes(object: &RawObject, namespace: &str) -> Result<Attributes, Error> {
    unimplemented!()
}

#[cfg(not(feature="prusti"))]
fn extract_attributes(object: &RawObject, namespace: &str) -> Result<Attributes, Error> {
    Ok(Attributes {
        height: object.height,

        connection_id: maybe_extract_attribute(object, &format!("{}.connection_id", namespace))
            .and_then(|val| val.parse().ok()),

        client_id: extract_attribute(object, &format!("{}.client_id", namespace))?
            .parse()
            .map_err(Error::parse)?,

        counterparty_connection_id: maybe_extract_attribute(
            object,
            &format!("{}.counterparty_connection_id", namespace),
        )
        .and_then(|val| val.parse().ok()),

        counterparty_client_id: extract_attribute(
            object,
            &format!("{}.counterparty_client_id", namespace),
        )?
        .parse()
        .map_err(Error::parse)?,
    })
}

impl Default for Attributes {
    fn default() -> Self {
        Attributes {
            height: Default::default(),
            connection_id: Default::default(),
            client_id: Default::default(),
            counterparty_connection_id: Default::default(),
            counterparty_client_id: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct OpenInit(Attributes);

impl OpenInit {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> &Option<ConnectionId> {
        &self.0.connection_id
    }
    pub fn height(&self) -> Height {
        self.0.height
    }
    pub fn set_height(&mut self, height: Height) {
        self.0.height = height;
    }
}

impl From<Attributes> for OpenInit {
    fn from(attrs: Attributes) -> Self {
        OpenInit(attrs)
    }
}

impl TryFrom<RawObject> for OpenInit {
    type Error = Error;
    fn try_from(obj: RawObject) -> Result<Self, Self::Error> {
        Ok(OpenInit(extract_attributes(&obj, "connection_open_init")?))
    }
}

impl From<OpenInit> for IbcEvent {
    fn from(v: OpenInit) -> Self {
        IbcEvent::OpenInitConnection(v)
    }
}

#[derive(Clone)]
pub struct OpenTry(Attributes);

impl OpenTry {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> &Option<ConnectionId> {
        &self.0.connection_id
    }
    pub fn height(&self) -> Height {
        self.0.height
    }
    pub fn set_height(&mut self, height: Height) {
        self.0.height = height;
    }
}

impl From<Attributes> for OpenTry {
    fn from(attrs: Attributes) -> Self {
        OpenTry(attrs)
    }
}

impl TryFrom<RawObject> for OpenTry {
    type Error = Error;
    fn try_from(obj: RawObject) -> Result<Self, Self::Error> {
        Ok(OpenTry(extract_attributes(&obj, "connection_open_try")?))
    }
}

impl From<OpenTry> for IbcEvent {
    fn from(v: OpenTry) -> Self {
        IbcEvent::OpenTryConnection(v)
    }
}

#[derive(Clone)]
pub struct OpenAck(Attributes);

impl OpenAck {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> &Option<ConnectionId> {
        &self.0.connection_id
    }
    pub fn height(&self) -> Height {
        self.0.height
    }
    pub fn set_height(&mut self, height: Height) {
        self.0.height = height;
    }
}

impl From<Attributes> for OpenAck {
    fn from(attrs: Attributes) -> Self {
        OpenAck(attrs)
    }
}

impl TryFrom<RawObject> for OpenAck {
    type Error = Error;
    fn try_from(obj: RawObject) -> Result<Self, Self::Error> {
        Ok(OpenAck(extract_attributes(&obj, "connection_open_ack")?))
    }
}

impl From<OpenAck> for IbcEvent {
    fn from(v: OpenAck) -> Self {
        IbcEvent::OpenAckConnection(v)
    }
}

#[derive(Clone)]
pub struct OpenConfirm(Attributes);

impl OpenConfirm {
    pub fn attributes(&self) -> &Attributes {
        &self.0
    }
    pub fn connection_id(&self) -> &Option<ConnectionId> {
        &self.0.connection_id
    }
    pub fn height(&self) -> Height {
        self.0.height
    }
    pub fn set_height(&mut self, height: Height) {
        self.0.height = height;
    }
}

impl From<Attributes> for OpenConfirm {
    fn from(attrs: Attributes) -> Self {
        OpenConfirm(attrs)
    }
}

impl TryFrom<RawObject> for OpenConfirm {
    type Error = Error;
    fn try_from(obj: RawObject) -> Result<Self, Self::Error> {
        Ok(OpenConfirm(extract_attributes(
            &obj,
            "connection_open_confirm",
        )?))
    }
}

impl From<OpenConfirm> for IbcEvent {
    fn from(v: OpenConfirm) -> Self {
        IbcEvent::OpenConfirmConnection(v)
    }
}
