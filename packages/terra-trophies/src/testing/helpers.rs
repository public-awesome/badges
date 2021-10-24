use cosmwasm_std::{StdError, StdResult};

pub fn assert_generic_error_message<T>(result: StdResult<T>, expected_msg: &str) {
    match result {
        Err(StdError::GenericErr {
            msg,
            ..
        }) => assert_eq!(msg, expected_msg),
        Err(other_err) => panic!("unexpected error: {:?}", other_err),
        Ok(_) => panic!("expected error but ok"),
    }
}
