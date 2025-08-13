use crate::{Item, Status};

pub fn cycle_status(item: &mut Item) {
    item.status = match item.status {
        Status::Todo => Status::Doing,
        Status::Doing => Status::Done,
        Status::Done => Status::Todo,
    };
}