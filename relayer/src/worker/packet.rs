use core::time::Duration;
use std::sync::{Arc, Mutex};
use crossbeam_channel::Receiver;
use tracing::{error, error_span, trace};
use ibc::Height;
use crate::chain::handle::ChainHandle;
use crate::event::monitor::EventBatch;
use crate::foreign_client::HasExpiredOrFrozenError;
use crate::link::Resubmit;
use crate::link::{error::LinkError, Link};
use crate::object::Packet;
use crate::telemetry;
use crate::util::task::{spawn_background_task, Next, TaskError, TaskHandle};
use super::error::RunError;
use super::WorkerCmd;
#[prusti_contracts::trusted]
fn handle_link_error_in_task(e: LinkError) -> TaskError<RunError> {
    if e.is_expired_or_frozen_error() {
        TaskError::Fatal(RunError::link(e))
    } else {
        TaskError::Ignore(RunError::link(e))
    }
}
/// Spawns a packet worker task in the background that handles the work of
/// processing pending txs between `ChainA` and `ChainB`.
#[prusti_contracts::trusted]
pub fn spawn_packet_worker<ChainA: ChainHandle, ChainB: ChainHandle>(
    path: Packet,
    link: Arc<Mutex<Link<ChainA, ChainB>>>,
    resubmit: Resubmit,
) -> TaskHandle {
    let span = {
        let relay_path = &link.lock().unwrap().a_to_b;
        error_span!(
            "packet", src_chain = % relay_path.src_chain().id(), src_port = % relay_path
            .src_port_id(), src_channel = % relay_path.src_channel_id(), dst_chain = %
            relay_path.dst_chain().id(),
        )
    };
    spawn_background_task(
        span,
        Some(Duration::from_millis(1000)),
        move || {
            handle_execute_schedule(&mut link.lock().unwrap(), &path, resubmit)?;
            Ok(Next::Continue)
        },
    )
}
#[prusti_contracts::trusted]
pub fn spawn_packet_cmd_worker<ChainA: ChainHandle, ChainB: ChainHandle>(
    cmd_rx: Receiver<WorkerCmd>,
    link: Arc<Mutex<Link<ChainA, ChainB>>>,
    mut should_clear_on_start: bool,
    clear_interval: u64,
    path: Packet,
) -> TaskHandle {
    let span = {
        let relay_path = &link.lock().unwrap().a_to_b;
        error_span!(
            "packet_cmd", src_chain = % relay_path.src_chain().id(), src_port = %
            relay_path.src_port_id(), src_channel = % relay_path.src_channel_id(),
            dst_chain = % relay_path.dst_chain().id(),
        )
    };
    spawn_background_task(
        span,
        Some(Duration::from_millis(200)),
        move || {
            if let Ok(cmd) = cmd_rx.try_recv() {
                handle_packet_cmd(
                    &mut link.lock().unwrap(),
                    &mut should_clear_on_start,
                    clear_interval,
                    &path,
                    cmd,
                )?;
            }
            Ok(Next::Continue)
        },
    )
}
/// Receives worker commands and handles them accordingly.
///
/// Given an `IbcEvent` command, updates the schedule and initiates
/// packet clearing if the `should_clear_on_start` flag has been toggled.
///
/// Given a `NewBlock` command, checks if packet clearing should occur
/// and performs it if so.
///
/// Given a `ClearPendingPackets` command, clears pending packets.
///
/// Regardless of the incoming command, this method also refreshes and
/// and executes any scheduled operational data that is ready.
#[prusti_contracts::trusted]
fn handle_packet_cmd<ChainA: ChainHandle, ChainB: ChainHandle>(
    link: &mut Link<ChainA, ChainB>,
    should_clear_on_start: &mut bool,
    clear_interval: u64,
    path: &Packet,
    cmd: WorkerCmd,
) -> Result<(), TaskError<RunError>> {
    let (do_clear, maybe_height) = match &cmd {
        WorkerCmd::IbcEvents { batch } => {
            if *should_clear_on_start {
                (true, Some(batch.height))
            } else {
                (false, None)
            }
        }
        WorkerCmd::NewBlock { height, .. } => {
            if *should_clear_on_start || should_clear_packets(clear_interval, *height) {
                (true, Some(*height))
            } else {
                (false, None)
            }
        }
        WorkerCmd::ClearPendingPackets => (true, None),
    };
    if do_clear {
        if *should_clear_on_start {
            *should_clear_on_start = false;
        }
        handle_clear_packet(link, clear_interval, path, maybe_height)?;
    }
    if let WorkerCmd::IbcEvents { batch } = cmd {
        handle_update_schedule(link, clear_interval, path, batch)
    } else {
        Ok(())
    }
}
/// Whether or not to clear pending packets at this `step` for some height.
/// If the relayer has been configured to clear packets on start and that has not
/// occurred yet, then packets are cleared.
///
/// If the specified height is reached, then packets are cleared if `clear_interval`
/// is not `0` and if we have reached the interval.
#[prusti_contracts::trusted]
fn should_clear_packets(clear_interval: u64, height: Height) -> bool {
    clear_interval != 0 && height.revision_height() % clear_interval == 0
}
#[prusti_contracts::trusted]
fn handle_update_schedule<ChainA: ChainHandle, ChainB: ChainHandle>(
    link: &mut Link<ChainA, ChainB>,
    clear_interval: u64,
    path: &Packet,
    batch: EventBatch,
) -> Result<(), TaskError<RunError>> {
    link.a_to_b.update_schedule(batch).map_err(handle_link_error_in_task)?;
    handle_execute_schedule(link, path, Resubmit::from_clear_interval(clear_interval))
}
#[prusti_contracts::trusted]
fn handle_clear_packet<ChainA: ChainHandle, ChainB: ChainHandle>(
    link: &mut Link<ChainA, ChainB>,
    clear_interval: u64,
    path: &Packet,
    height: Option<Height>,
) -> Result<(), TaskError<RunError>> {
    link.a_to_b.schedule_packet_clearing(height).map_err(handle_link_error_in_task)?;
    handle_execute_schedule(link, path, Resubmit::from_clear_interval(clear_interval))
}
#[prusti_contracts::trusted]
fn handle_execute_schedule<ChainA: ChainHandle, ChainB: ChainHandle>(
    link: &mut Link<ChainA, ChainB>,
    _path: &Packet,
    resubmit: Resubmit,
) -> Result<(), TaskError<RunError>> {
    link.a_to_b.refresh_schedule().map_err(handle_link_error_in_task)?;
    link.a_to_b
        .execute_schedule()
        .map_err(|e| {
            if e.is_expired_or_frozen_error() {
                TaskError::Fatal(RunError::link(e))
            } else {
                error!("will retry: schedule execution encountered error: {}", e,);
                TaskError::Ignore(RunError::link(e))
            }
        })?;
    let summary = link.a_to_b.process_pending_txs(resubmit);
    if !summary.is_empty() {
        trace!("produced relay summary: {:?}", summary);
    }
    telemetry!(packet_metrics(_path, & summary));
    Ok(())
}
#[cfg(feature = "telemetry")]
use crate::link::RelaySummary;
#[cfg(feature = "telemetry")]
#[prusti_contracts::trusted]
fn packet_metrics(path: &Packet, summary: &RelaySummary) {
    receive_packet_metrics(path, summary);
    acknowledgment_metrics(path, summary);
    timeout_metrics(path, summary);
}
#[cfg(feature = "telemetry")]
#[prusti_contracts::trusted]
fn receive_packet_metrics(path: &Packet, summary: &RelaySummary) {
    use ibc::events::IbcEvent::WriteAcknowledgement;
    let count = summary
        .events
        .iter()
        .filter(|e| matches!(e, WriteAcknowledgement(_)))
        .count();
    telemetry!(
        ibc_receive_packets, & path.src_chain_id, & path.src_channel_id, & path
        .src_port_id, count as u64,
    );
}
#[cfg(feature = "telemetry")]
#[prusti_contracts::trusted]
fn acknowledgment_metrics(path: &Packet, summary: &RelaySummary) {
    use ibc::events::IbcEvent::AcknowledgePacket;
    let count = summary
        .events
        .iter()
        .filter(|e| matches!(e, AcknowledgePacket(_)))
        .count();
    telemetry!(
        ibc_acknowledgment_packets, & path.src_chain_id, & path.src_channel_id, & path
        .src_port_id, count as u64,
    );
}
#[cfg(feature = "telemetry")]
#[prusti_contracts::trusted]
fn timeout_metrics(path: &Packet, summary: &RelaySummary) {
    use ibc::events::IbcEvent::TimeoutPacket;
    let count = summary.events.iter().filter(|e| matches!(e, TimeoutPacket(_))).count();
    telemetry!(
        ibc_timeout_packets, & path.src_chain_id, & path.src_channel_id, & path
        .src_port_id, count as u64,
    );
}

