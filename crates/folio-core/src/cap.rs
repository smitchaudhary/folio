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
                let removed = inbox.remove(pos);
                inbox.push(new_item);
                Ok((inbox, vec![removed]))
            } else {
                Err(CapError::Full)
            }
        }
        OverflowStrategy::Any => {
            if !inbox.is_empty() {
                let removed = inbox.remove(0);
                inbox.push(new_item);
                Ok((inbox, vec![removed]))
            } else {
                Err(CapError::Full)
            }
        }
    }
}
