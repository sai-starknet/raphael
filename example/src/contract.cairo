use starknet::{ContractAddress, get_tx_info};


fn get_origin_address() -> ContractAddress {
    get_tx_info().deref().account_contract_address
}

#[starknet::interface]
trait IContract<TContractState> {}

#[starknet::contract]
mod my_contract {
    use super::{IContract};
    use starknet::{ContractAddress, get_caller_address};
    use starknet::storage::{Map};

    use sai_owners_writers::{owners_writers_component, OwnersWriters};

    component!(path: owners_writers_component, storage: owners_writers, event: OwnersWritersEvents);


    #[storage]
    struct Storage {
        #[substorage(v0)]
        owners_writers: owners_writers_component::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        #[flat]
        OwnersWritersEvents: owners_writers_component::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.grant_owner(owner);
    }
}
