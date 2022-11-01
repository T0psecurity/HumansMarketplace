use crate::execute::AskInfo;
#[cfg(test)]
use crate::execute::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg, };
use crate::query::{ query_ask,  query_bids, query_asks_by_bid_count, query_all_bids, query_asks_sorted_by_expiration};
use crate::state:: SaleType;
use crate::helpers::ExpiryRange;

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{ Addr, DepsMut,Uint128,to_binary,CosmosMsg, WasmMsg,  Coin, BankMsg};
use cw721::{Cw721ReceiveMsg,Cw721ExecuteMsg};


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

        create_collection_address: "create_collection_address".to_string()
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
        create_collection_address: "create_collection_address".to_string()
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

#[test]
fn test_accept_bid_without_bid(){
  let mut deps = mock_dependencies();
  let mut env = mock_env();
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

  env.block.time = env.block.time.plus_seconds(350);

  let info = mock_info("seller1", &[]);
  let msg = ExecuteMsg::AcceptBid  { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  assert_eq!(res.messages.len(), 1);
  assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(WasmMsg::Execute { 
    contract_addr: "collection1".to_string(), 
    msg: to_binary(&Cw721ExecuteMsg::TransferNft { 
      recipient: "seller1".to_string(), 
      token_id: "Test.1".to_string() 
    }).unwrap(), 
    funds: vec![] 
  }));
}


#[test]
fn test_asks_sort_by_expiration(){
  let mut deps = mock_dependencies();
  let mut env = mock_env();
  setup_contract(deps.as_mut());

  let sell_msg = AskInfo{
    sale_type: SaleType::Auction,
    collection: Addr::unchecked("collection1".to_string()),
    token_id: "Test.1".to_string(),
    price: Coin { denom: "uheart".to_string(), amount: Uint128::new(300) },
    funds_recipient: None,
    expires: 150,
  };

  let info = mock_info("collection1", &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender: "seller1".to_string(),
      token_id: "Test.1".to_string(),
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

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

 let sell_msg = AskInfo{
    sale_type: SaleType::Auction,
    collection: Addr::unchecked("collection3".to_string()),
    token_id: "Test.3".to_string(),
    price: Coin { denom: "uheart".to_string(), amount: Uint128::new(300) },
    funds_recipient: None,
    expires: 200,
  };

  let info = mock_info("collection3", &[]);
  let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg{
      sender: "seller3".to_string(),
      token_id: "Test.3".to_string(),
      msg:to_binary(&sell_msg).unwrap()
  });
  execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  env.block.time = env.block.time.plus_seconds(170);

  let asks = query_asks_sorted_by_expiration(deps.as_ref(), env.clone(), Some(1)).unwrap();
  println!("asks {:?}", asks)
}


#[test]
fn test_remove_ask() {
  let mut deps = mock_dependencies();
  let mut env = mock_env();
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

  let info = mock_info("seller1", &[]);
  let msg = ExecuteMsg::RemoveAsk { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

  assert_eq!(res.messages.len(), 2);
  assert_eq!(res.messages[0].msg, CosmosMsg::Wasm(WasmMsg::Execute{
    contract_addr: "collection1".to_string(),
    msg: to_binary(&Cw721ExecuteMsg::TransferNft{
      recipient: "seller1".to_string(),
      token_id: "Test.1".to_string()
    }).unwrap(),  
    funds: vec![]
  }));

  assert_eq!(res.messages[1].msg, CosmosMsg::Bank(BankMsg::Send { to_address: "bider1".to_string(), amount: vec![Coin{
      denom: "uheart".to_string(),
      amount: Uint128::new(350)
  }] }));

  let bid = query_bids(deps.as_ref(), Addr::unchecked("collection1"), "Test.1".to_string(), None, None).unwrap();
  println!("bid {:?}", bid)
}

#[test]
fn test_remove_bids(){
  let mut deps = mock_dependencies();
  let mut env = mock_env();
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
  let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
  println!("bid message length compare");
  assert_eq!(res.messages.len(),1);

  let bids = query_bids(deps.as_ref(), Addr::unchecked("collection1") , "Test.1".to_string(), None, Some(1)).unwrap();
  println!("bids by pagination {:?}",bids);

  let all_bids = query_all_bids(deps.as_ref(), Addr::unchecked("collection1") , "Test.1".to_string()).unwrap();
  println!("all bids {:?}", all_bids);

  let ask = query_ask(deps.as_ref(), Addr::unchecked("collection1"), "Test.1".to_string()).unwrap();
  println!("{:?}",ask)

  // env.block.time = env.block.time.plus_seconds(350);

  // let info = mock_info("seller1", &[]);
  // let msg = ExecuteMsg::AcceptBid { collection: "collection1".to_string(), token_id: "Test.1".to_string() };
  // let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

  // println!("compare bids messges length{:?}", res.messages.len());
  // assert_eq!(res.messages.len(), 4);
  // assert_eq!(res.messages[0].msg, CosmosMsg::Bank(BankMsg::Send{
  //   to_address: "owner1".to_string(),
  //   amount: vec![Coin{
  //     denom: "uheart".to_string(),
  //     amount: Uint128::new(40)
  //   }]
  // }));
  // assert_eq!(res.messages[1].msg, CosmosMsg::Bank(BankMsg::Send{
  //   to_address: "seller1".to_string(),
  //   amount: vec![Coin{
  //     denom: "uheart".to_string(),
  //     amount: Uint128::new(360)
  //   }]
  // }));
  // assert_eq!(res.messages[2].msg, CosmosMsg::Wasm(WasmMsg::Execute{
  //   contract_addr: "collection1".to_string(),
  //   msg: to_binary(&Cw721ExecuteMsg::TransferNft{
  //     recipient: "bider2".to_string(),
  //     token_id: "Test.1".to_string()
  //   }).unwrap(),
  //   funds: vec![]
  // }));

  // let all_bids = query_all_bids(deps.as_ref(), Addr::unchecked("collection1") , "Test.1".to_string()).unwrap();
  // println!("all bids after accept the max bid {:?}", all_bids);
}