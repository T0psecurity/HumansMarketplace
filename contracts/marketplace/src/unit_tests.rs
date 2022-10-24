use crate::execute::AskInfo;
#[cfg(test)]
use crate::execute::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg, };
use crate::query::{ query_ask,  query_bids, query_asks_by_bid_count};
use crate::state:: SaleType;
use crate::helpers::ExpiryRange;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr, DepsMut, Timestamp, Uint128,to_binary, Env, Decimal, CosmosMsg, WasmMsg, StdResult, Response, Coin, BankMsg};
use cw721::{Cw721ReceiveMsg,Cw721ExecuteMsg};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg};

fn setup_contract(deps: DepsMut){
   let instantiate_msg = InstantiateMsg {
        trading_fee_bps: 25,
        /// Valid time range for Asks
        /// (min, max) in seconds
        ask_expiry: ExpiryRange{
          min:100,
          max:500
        },
        /// Valid time range for Bids
        /// (min, max) in seconds
        bid_expiry: ExpiryRange{
          min: 100,
          max: 500
        },
        /// Operators are entites that are responsible for maintaining the active state of Asks.
        /// They listen to NFT transfer events, and update the active state of Asks.
        operators: vec![],
        /// The address of the airdrop claim contract to detect sales
        sale_hook: Some("hook".to_string()),
        /// Max basis points for the finders fee
        //  max_finders_fee_bps: u64,
        /// Min value for bids and asks
        min_price: Uint128::new(10),
        /// Listing fee to reduce spam
        listing_fee: Uint128::zero(),
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps, mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(res.messages.len(), 0)
}



#[test]
fn init_contract() {
    let mut deps = mock_dependencies();
    let instantiate_msg = InstantiateMsg {
         trading_fee_bps: 25,
        /// Valid time range for Asks
        /// (min, max) in seconds
        ask_expiry: ExpiryRange{
          min:100,
          max:500
        },
        /// Valid time range for Bids
        /// (min, max) in seconds
        bid_expiry: ExpiryRange{
          min: 100,
          max: 500
        },
        /// Operators are entites that are responsible for maintaining the active state of Asks.
        /// They listen to NFT transfer events, and update the active state of Asks.
        operators: vec![],
        /// The address of the airdrop claim contract to detect sales
        sale_hook: Some("hook".to_string()),
        /// Max basis points for the finders fee
        //  max_finders_fee_bps: u64,
        /// Min value for bids and asks
        min_price: Uint128::new(10),
        /// Listing fee to reduce spam
        listing_fee: Uint128::zero(),
    };
    let info = mock_info("owner", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
    assert_eq!(0, res.messages.len());
   
}


#[test]
fn test_ask(){
  let mut deps = mock_dependencies();
  let env = mock_env();
  setup_contract(deps.as_mut());

  let sell_msg = AskInfo{
    sale_type: SaleType::Auction,
    collection: Addr::unchecked("collection1".to_string()),
    token_id: "Test.1".to_string(),
    price: Coin { denom: "uheart".to_string(), amount: Uint128::new(300) },
    funds_recipient: None,
    expires: 300,
  };

  let info = mock_info("collection1", &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender: "seller1".to_string(),
      token_id: "Test.1".to_string(),
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  
  let info = mock_info("bider1", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(350)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();

  let info = mock_info("bider2", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(400)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();

  let info = mock_info("bider3", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(450)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();

 

  let _bids = query_bids(
    deps.as_ref(), 
    Addr::unchecked("collection1".to_string()), 
    "Test.1".to_string(), 
    None, 
    Some(20)
  ).unwrap();
 

  //Second sale
  
  let sell_msg = AskInfo{
    sale_type: SaleType::Auction,
    collection: Addr::unchecked("collection2".to_string()),
    token_id: "Test.2".to_string(),
    price: Coin { denom: "uheart".to_string(), amount: Uint128::new(300) },
    funds_recipient: None,
    expires: 300,
  };

  let info = mock_info("collection2", &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender: "seller2".to_string(),
      token_id: "Test.2".to_string(),
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  
  let info = mock_info("bider1", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(350)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection2".to_string(), token_id: "Test.2".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();



  //Third sale
  
  let sell_msg = AskInfo{
    sale_type: SaleType::Auction,
    collection: Addr::unchecked("collection3".to_string()),
    token_id: "Test.3".to_string(),
    price: Coin { denom: "uheart".to_string(), amount: Uint128::new(300) },
    funds_recipient: None,
    expires: 300,
  };

  let info = mock_info("collection3", &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender: "seller3".to_string(),
      token_id: "Test.3".to_string(),
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  
  let info = mock_info("bider1", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(350)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection3".to_string(), token_id: "Test.3".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();

  let info = mock_info("bider2", &[Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(400)
  }]);
  let msg = ExecuteMsg::SetBid { collection: "collection3".to_string(), token_id: "Test.3".to_string() };
  execute(deps.as_mut(), env.clone(), info, msg).unwrap();
  
  let ask_info = query_ask(deps.as_ref(), Addr::unchecked("collection1".to_string()), "Test.1".to_string()).unwrap();
  println!("collection1 {:?}", ask_info.ask.unwrap().bid_count);
  
  let ask_info = query_ask(deps.as_ref(), Addr::unchecked("collection2".to_string()), "Test.2".to_string()).unwrap();
  println!("collection2 {:?}", ask_info.ask.unwrap().bid_count);
 
  let ask_info = query_ask(deps.as_ref(), Addr::unchecked("collection3".to_string()), "Test.3".to_string()).unwrap();
  println!("collection3 {:?}", ask_info.ask.unwrap().bid_count);


  let result = query_asks_by_bid_count(deps.as_ref(), None, Some(20)).unwrap();
  println!("{:?}",result)
}