#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, BankMsg, has_coins};
use cosmwasm_std::Addr;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, AddressResponse, CostResponse, OwnerResponse};
use crate::state::{State, STATE};


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


// instantiate
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        // set initial contract values
        address : msg.address,
        rentcost : msg.rentcost,
        renters : Vec::new(),
        owner : info.sender.clone(),
        ownername : msg.ownername,
    };

    // set contract ver and save contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    // provides response showing contract was initialized
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("address", msg.address))
}

// execute methods
#[cfg_attr(not(feature = "library"), entry_point)]

// base execution
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // matches to each func
    match msg {
        ExecuteMsg::RenterAdd {} => try_add(deps, env, info),
        ExecuteMsg::RenterPay {} => try_pay(deps, env, info),
        ExecuteMsg::RenterBoot {} => try_boot(deps, info),
        ExecuteMsg::ChangeRent {newprice} => try_ChangeRent(deps, info, newprice),
    }
}

// function for landlord to accept a renter
pub fn try_add(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    // updates the state of the contract
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // if the user is not the owner, they will not be able to change the contract
        state.renters.push(env.contract.address);
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_add"))
}

pub fn try_pay(
    deps: DepsMut, 
    env: Env, 
    info: MessageInfo,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> { 
        // one critical error here.
        if has_coins(&info.funds, &state.rentcost){
            let send = BankMsg::Send {
                to_address: state.owner.into_string(),
                amount: info.funds,
            };
        }
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "try_pay"))
}


// function for a landlord to evict a renter from the property
pub fn try_boot(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized{});
        }
        state.renters.pop();
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_boot"))

}

// function for a landlord to change the price of rent
pub fn try_ChangeRent(deps: DepsMut, info: MessageInfo, newprice: Vec<Coin>) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized{});
        }
        // need to change to coin vector
        state.rentcost = newprice;
        Ok(state)
    })?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddress {} => to_binary(&query_address(deps)?),
        QueryMsg::GetRentCost {} => to_binary(&query_rent(deps)?),
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
    }
}

fn query_address(deps: Deps) -> StdResult<AddressResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(AddressResponse { address: state.address })
}

fn query_rent(deps: Deps) -> StdResult<CostResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CostResponse {rentcost: state.rentcost})
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse {ownername : state.ownername})
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    // initialization test
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg { address: String::from("33 pawn lane"), rentcost: coins(10, "luna"), renters : Vec::new(), ownername: String::from("john doe") };
        let info = mock_info("landlord", &coins(0, "luna"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(0, res.messages.len());

        // query to check the address is valid
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAddress {}).unwrap();
        let value: AddressResponse = from_binary(&res).unwrap();
        assert_eq!(String::from("33 pawn lane"), value.address);

        // query to check the cost is valid
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetRentCost {}).unwrap();
        let value: CostResponse = from_binary(&res).unwrap();
        assert_eq!(coins(10, "luna"), value.rentcost);

        // query to check that the owner's name is valid
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetRentCost {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(String::from("john doe"), value.ownername);
    }

    #[test]
    fn renters() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let msg = InstantiateMsg { address: String::from("33 pawn lane"), rentcost: coins(10, "luna"), renters : Vec::new(), ownername: String::from("john doe") };
        let info = mock_info("landlord", &coins(0, "luna"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info2 = mock_info("jane doe", &coins(2, "luna"));
        let msg = ExecuteMsg::RenterAdd {};
        let _res = execute(deps.as_mut(), mock_env(), info2, msg).unwrap();
        
        // shouldnt be able to remove jane doe
        let info3 = mock_info("jaja doe", &coins(3, "luna"));
        let msg = ExecuteMsg::RenterBoot {};
        let res = execute(deps.as_mut(), mock_env(), info3, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

    }
    
    
}
