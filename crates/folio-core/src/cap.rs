use crate::{CapError, Item, OverflowStrategy, Status};

pub fn add_with_cap(
    mut inbox: Vec<Item>,
    new_item: Item,
    max: usize,
    strat: OverflowStrategy,
) -> Result<(Vec<Item>, Vec<Item>), CapError> {
    if inbox.len() < max {
        inbox.push(new_item);
        return Ok((inbox, vec![]));
    }

    match strat {
        OverflowStrategy::Abort => Err(CapError::Full),
        OverflowStrategy::Todo => {
            if let Some(pos) = inbox.iter().position(|i| i.status == Status::Todo) {
                let mut removed = inbox.remove(pos);
                removed.status = Status::Done;
                crate::status::update_timestamps(&mut removed);
                inbox.push(new_item);
                Ok((inbox, vec![removed]))
            } else {
                Err(CapError::Full)
            }
        }
        OverflowStrategy::Any => {
            if !inbox.is_empty() {
                let mut removed = inbox.remove(0);
                removed.status = Status::Done;
                crate::status::update_timestamps(&mut removed);
                inbox.push(new_item);
                Ok((inbox, vec![removed]))
            } else {
                Err(CapError::Full)
            }
        }
    }
}
