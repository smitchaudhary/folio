use crate::{Item, Status};
use chrono::Utc;

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
