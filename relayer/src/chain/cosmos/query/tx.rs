use ibc::core::ics02_client::events as ClientEvents;
use ibc::core::ics04_channel::events as ChannelEvents;
use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics24_host::identifier::ChainId;
use ibc::events::{from_tx_response_event, IbcEvent};
use ibc::Height as ICSHeight;
use tendermint::abci::transaction::Hash as TxHash;
use tendermint::abci::Event;
use tendermint_rpc::endpoint::tx::Response as TxResponse;
use tendermint_rpc::{Client, HttpClient, Order, Url};
use crate::chain::cosmos::query::{header_query, packet_query, tx_hash_query};
use crate::chain::requests::{
    QueryClientEventRequest, QueryHeight, QueryPacketEventDataRequest, QueryTxHash,
    QueryTxRequest,
};
use crate::error::Error;
/// This function queries transactions for events matching certain criteria.
/// 1. Client Update request - returns a vector with at most one update client event
/// 2. Packet event request - returns at most one packet event for each sequence specified
///    in the request.
///    Note - there is no way to format the packet query such that it asks for Tx-es with either
///    sequence (the query conditions can only be AND-ed).
///    There is a possibility to include "<=" and ">=" conditions but it doesn't work with
///    string attributes (sequence is emmitted as a string).
///    Therefore, for packets we perform one tx_search for each sequence.
///    Alternatively, a single query for all packets could be performed but it would return all
///    packets ever sent.
#[prusti_contracts::trusted]
pub async fn query_txs(
    chain_id: &ChainId,
    rpc_client: &HttpClient,
    rpc_address: &Url,
    request: QueryTxRequest,
) -> Result<Vec<IbcEvent>, Error> {
    crate::time!("query_txs");
    crate::telemetry!(query, chain_id, "query_txs");
    match request {
        QueryTxRequest::Packet(request) => {
            crate::time!("query_txs: query packet events");
            let mut result: Vec<IbcEvent> = vec![];
            for seq in &request.sequences {
                let response = rpc_client
                    .tx_search(
                        packet_query(&request, *seq),
                        false,
                        1,
                        1,
                        Order::Ascending,
                    )
                    .await
                    .map_err(|e| Error::rpc(rpc_address.clone(), e))?;
                assert!(
                    response.txs.len() <= 1,
                    "packet_from_tx_search_response: unexpected number of txs"
                );
                if response.txs.is_empty() {
                    continue;
                }
                if let Some(event)
                    = packet_from_tx_search_response(
                        chain_id,
                        &request,
                        *seq,
                        response.txs[0].clone(),
                    )? {
                    result.push(event);
                }
            }
            Ok(result)
        }
        QueryTxRequest::Client(request) => {
            crate::time!("query_txs: single client update event");
            let mut response = rpc_client
                .tx_search(header_query(&request), false, 1, 1, Order::Ascending)
                .await
                .map_err(|e| Error::rpc(rpc_address.clone(), e))?;
            if response.txs.is_empty() {
                return Ok(vec![]);
            }
            assert!(
                response.txs.len() <= 1,
                "packet_from_tx_search_response: unexpected number of txs"
            );
            let tx = response.txs.remove(0);
            let event = update_client_from_tx_search_response(chain_id, &request, tx)?;
            Ok(event.into_iter().collect())
        }
        QueryTxRequest::Transaction(tx) => {
            let mut response = rpc_client
                .tx_search(tx_hash_query(&tx), false, 1, 1, Order::Ascending)
                .await
                .map_err(|e| Error::rpc(rpc_address.clone(), e))?;
            if response.txs.is_empty() {
                Ok(vec![])
            } else {
                let tx = response.txs.remove(0);
                Ok(all_ibc_events_from_tx_search_response(chain_id, tx))
            }
        }
    }
}
#[prusti_contracts::trusted]
fn update_client_from_tx_search_response(
    chain_id: &ChainId,
    request: &QueryClientEventRequest,
    response: TxResponse,
) -> Result<Option<IbcEvent>, Error> {
    let height = ICSHeight::new(chain_id.version(), u64::from(response.height))
        .map_err(|_| Error::invalid_height_no_source())?;
    if let QueryHeight::Specific(specific_query_height) = request.query_height {
        if height > specific_query_height {
            return Ok(None);
        }
    }
    Ok(
        response
            .tx_result
            .events
            .into_iter()
            .filter(|event| event.type_str == request.event_id.as_str())
            .flat_map(|event| ClientEvents::try_from_tx(&event))
            .flat_map(|event| match event {
                IbcEvent::UpdateClient(mut update) => {
                    update.common.height = height;
                    Some(update)
                }
                _ => None,
            })
            .find(|update| {
                update.common.client_id == request.client_id
                    && update.common.consensus_height == request.consensus_height
            })
            .map(IbcEvent::UpdateClient),
    )
}
#[prusti_contracts::trusted]
fn packet_from_tx_search_response(
    chain_id: &ChainId,
    request: &QueryPacketEventDataRequest,
    seq: Sequence,
    response: TxResponse,
) -> Result<Option<IbcEvent>, Error> {
    let height = ICSHeight::new(chain_id.version(), u64::from(response.height))
        .map_err(|_| Error::invalid_height_no_source())?;
    if let QueryHeight::Specific(query_height) = request.height {
        if height > query_height {
            return Ok(None);
        }
    }
    Ok(
        response
            .tx_result
            .events
            .into_iter()
            .find_map(|ev| filter_matching_event(ev, request, seq)),
    )
}
#[prusti_contracts::trusted]
fn filter_matching_event(
    event: Event,
    request: &QueryPacketEventDataRequest,
    seq: Sequence,
) -> Option<IbcEvent> {
    fn matches_packet(
        request: &QueryPacketEventDataRequest,
        seq: Sequence,
        packet: &Packet,
    ) -> bool {
        packet.source_port == request.source_port_id
            && packet.source_channel == request.source_channel_id
            && packet.destination_port == request.destination_port_id
            && packet.destination_channel == request.destination_channel_id
            && packet.sequence == seq
    }
    if event.type_str != request.event_id.as_str() {
        return None;
    }
    let ibc_event = ChannelEvents::try_from_tx(&event)?;
    match ibc_event {
        IbcEvent::SendPacket(
            ref send_ev,
        ) if matches_packet(request, seq, &send_ev.packet) => Some(ibc_event),
        IbcEvent::WriteAcknowledgement(
            ref ack_ev,
        ) if matches_packet(request, seq, &ack_ev.packet) => Some(ibc_event),
        _ => None,
    }
}
#[prusti_contracts::trusted]
pub async fn query_tx_response(
    rpc_client: &HttpClient,
    rpc_address: &Url,
    tx_hash: &TxHash,
) -> Result<Option<TxResponse>, Error> {
    let response = rpc_client
        .tx_search(tx_hash_query(&QueryTxHash(*tx_hash)), false, 1, 1, Order::Ascending)
        .await
        .map_err(|e| Error::rpc(rpc_address.clone(), e))?;
    Ok(response.txs.into_iter().next())
}
#[prusti_contracts::trusted]
fn all_ibc_events_from_tx_search_response(
    chain_id: &ChainId,
    response: TxResponse,
) -> Vec<IbcEvent> {
    let height = ICSHeight::new(chain_id.version(), u64::from(response.height)).unwrap();
    let deliver_tx_result = response.tx_result;
    if deliver_tx_result.code.is_err() {
        vec![
            IbcEvent::ChainError(format!("deliver_tx for {} reports error: code={:?}, log={:?}",
            response.hash, deliver_tx_result.code, deliver_tx_result.log))
        ]
    } else {
        let result = deliver_tx_result
            .events
            .iter()
            .flat_map(|event| from_tx_response_event(height, event).into_iter())
            .collect::<Vec<_>>();
        result
    }
}

