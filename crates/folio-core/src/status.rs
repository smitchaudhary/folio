use crate::{Item, Status};
use chrono::Utc;

#[derive(Debug, Clone, PartialEq)]
pub struct StatusTransitionResult {
    pub old_status: Status,
    pub new_status: Status,
    pub should_archive: bool,
    pub should_move_to_inbox: bool,
    pub status_changed: bool,
}

pub fn change_item_status(item: &mut Item, new_status: Status) -> StatusTransitionResult {
    let old_status = item.status.clone();
    let status_changed = old_status != new_status;

    item.status = new_status.clone();

    update_timestamps(item);

    let should_archive = old_status != Status::Done && new_status == Status::Done;
    let should_move_to_inbox = old_status == Status::Done && new_status != Status::Done;

    StatusTransitionResult {
        old_status,
        new_status,
        should_archive,
        should_move_to_inbox,
        status_changed,
    }
}

pub fn update_timestamps(item: &mut Item) {
    match item.status {
        Status::Todo => {
            item.started_at = None;
            item.finished_at = None;
        }
        Status::Doing => {
            if item.started_at.is_none() {
                item.started_at = Some(Utc::now());
            }
        }
        Status::Done => {
            if item.finished_at.is_none() {
                item.finished_at = Some(Utc::now());
            }
        }
    }
}
