#![cfg(test)]

use crate::contract::instantiate;
use crate::ibc::{ibc_channel_connect, ibc_channel_open, ICS20_ORDERING, ICS20_VERSION};

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    DepsMut, IbcChannel, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcEndpoint, OwnedDeps,
};

use crate::msg::{AllowMsg, InitMsg};

pub const DEFAULT_TIMEOUT: u64 = 3600; // 1 hour,
pub const CONTRACT_PORT: &str = "ibc:wasm1234567890abcdef";
pub const REMOTE_PORT: &str = "transfer";
pub const CONNECTION_ID: &str = "connection-2";

pub fn mock_channel(channel_id: &str) -> IbcChannel {
    IbcChannel::new(
        IbcEndpoint {
            port_id: CONTRACT_PORT.into(),
            channel_id: channel_id.into(),
        },
        IbcEndpoint {
            port_id: REMOTE_PORT.into(),
            channel_id: format!("{}5", channel_id),
        },
        ICS20_ORDERING,
        ICS20_VERSION,
        CONNECTION_ID,
    )
}

// we simulate instantiate and ack here
pub fn add_channel(mut deps: DepsMut, channel_id: &str) {
    let channel = mock_channel(channel_id);
    let open_msg = IbcChannelOpenMsg::new_init(channel.clone());
    ibc_channel_open(deps.branch(), mock_env(), open_msg).unwrap();
    let connect_msg = IbcChannelConnectMsg::new_ack(channel, ICS20_VERSION);
    ibc_channel_connect(deps.branch(), mock_env(), connect_msg).unwrap();
}

pub fn setup(
    channels: &[&str],
    allow: &[(&str, &str, u64)],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    let allowlist = allow
        .iter()
        .map(|(contract, code_hash, gas)| AllowMsg {
            contract: contract.to_string(),
            code_hash: code_hash.to_string(),
            gas_limit: Some(*gas),
            denom: String::from("uscrt"),
            port: String::from("wasm.aaaa"),
        })
        .collect();

    // instantiate an empty contract
    let instantiate_msg = InitMsg {
        admin: "gov".to_string(),
        allowlist,
    };
    let info = mock_info(&String::from("anyone"), &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg.clone()).unwrap();
    print!("======== {:?}", instantiate_msg.clone());
    assert_eq!(1, res.messages.len());

    for channel in channels {
        add_channel(deps.as_mut(), channel);
    }
    deps
}
