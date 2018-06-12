
#[macro_use]
extern crate nom;

use nom::IResult;

use std::str::FromStr;
use std::result::Result;
use std::convert::Into;

#[cfg(test)]
mod tests {
    //use super::parse_auction_summary;
    use super::AuctionSummaryMsg;
    use super::AuctionUpdateMsg;
    use super::AddOrderMsg;
    use super::BATSMsgFactory;
    use std::env;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn test_parse_auction_summary() {
        let msg = "28800168JAAPLSPOTC00010068000000020000";
        let res = AuctionSummaryMsg::parse_msg(msg);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn test_parse_add_order() {
        let msg = "28800168A1K27GA00000YS000100AAPL  0001831900Y";
        let msg_long = "28800169d1K27GA00000YS000100AAPL  0001831900YBAML";

        let res = AddOrderMsg::parse_msg(msg);
        println!("{:?}", res);
        assert!(res.is_ok());

        let o = res.unwrap();        
        assert_eq!(o.timestamp, 28800168);
        assert_eq!( o.msg_type, 'A');
        assert_eq!( o.order_id,  204969015920664610);
        assert_eq!( o.side,     'S');
        assert_eq!( o.shares,   100);
        assert_eq!( o.symbol,   "AAPL  ");
        assert_eq!( o.price,    1831900);
        assert_eq!( o.display,  'Y');

        let res = AddOrderMsg::parse_msg(msg_long);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

        #[test]
    fn test_parse_auction_update() {
        let msg = "28800168IAAPLSPOTC00010068000000020000000001000000015034000001309800"; 
        let res = AuctionUpdateMsg::parse_msg(msg);
        println!("{:?}", res);
        assert!(res.is_ok());    
    }
    
    #[test]
    fn test_parse_file() {
        let path = env::current_dir().unwrap();
        
        println!("The current directory is {}", path.display());
        let filename = "src/pitch_example_data";
        let f = File::open(filename).expect("file not found");
        let f = BufReader::new(f);

        for line in f.lines() {
            let msg = line.unwrap();
            let buf = msg.as_str();
            if buf.chars().nth(8) == Some('J') {
                let res = AuctionSummaryMsg::parse_msg(buf);
                println!("{:?}", res);
                assert!(res.is_ok());
                break;
            }
        }
    }

    #[test]
    fn test_factory() {
        let obj = BATSMsgFactory::parse("28800168A1K27GA00000YS000100AAPL  0001831900Y");
        println!("Return result from msg factory {:?}", obj);
        let msg_obj : Option<AddOrderMsg> = obj.into();
        assert!(msg_obj.is_some());
        println!("After into {:?}", msg_obj);

        let obj = BATSMsgFactory::parse("28800168JAAPLSPOTC00010068000000020000");
        println!("Return result from msg factory {:?}", obj);
        let msg_obj : Option<AuctionSummaryMsg> = obj.into();
        println!("After into {:?}", msg_obj);
        assert!(msg_obj.is_some());
    }
}

macro_rules! create_into_function {
    ($objname : ident) => (
        impl Into<Option<$objname>> for BATSMessage {
            fn into(self) -> Option<$objname> {
                match self {
                    BATSMessage::$objname(u) => Some(u), 
                    _ => None,
                }
            }
        }  
    )
}

macro_rules! create_parse_impl {
    ($objname : ident, $parse_func : ident) => (
        impl $objname {
            pub fn parse_msg( msg : &str ) -> Result<$objname, nom::Err<&str>> {
                let o = $parse_func(msg);
                if o.is_ok() {
                    Ok(o.unwrap().1)
                } else {
                    Err(o.unwrap_err())
                }
            }
        }
    )
}

#[derive(Debug)]
pub enum BATSMessage { // For implementing message factory
    AuctionSummaryMsg(AuctionSummaryMsg), 
    AddOrderMsg(AddOrderMsg),
    AuctionUpdateMsg(AuctionUpdateMsg),
}

// use macros to generate into functions for all msgs
create_into_function!(AddOrderMsg);
create_into_function!(AuctionSummaryMsg);
create_into_function!(AuctionUpdateMsg);

// use macros to generate impl parse_msg functions for all msgs
create_parse_impl!(AddOrderMsg, parse_add_order);
create_parse_impl!(AuctionSummaryMsg, parse_auction_summary);
create_parse_impl!(AuctionUpdateMsg, parse_auction_update);

pub struct BATSMsgFactory {} // this coupled with impl below makes it like a 
                             // factory method exposed via a static class method.
impl BATSMsgFactory {
    pub fn parse( msg : &str ) -> BATSMessage {
        let code = &msg[8..9];
        let obj = match code {
            "A" => BATSMessage::AddOrderMsg( AddOrderMsg::parse_msg(msg).unwrap() ), 
            "d" => BATSMessage::AddOrderMsg( AddOrderMsg::parse_msg(msg).unwrap() ),
            "J" => BATSMessage::AuctionSummaryMsg( AuctionSummaryMsg::parse_msg(msg).unwrap() ),
            "I" => BATSMessage::AuctionUpdateMsg( AuctionUpdateMsg::parse_msg(msg).unwrap() ),
            &_ => unimplemented!(),
        };
        obj
    }
}

#[derive(Debug)]
pub struct AuctionSummaryMsg {
    timestamp    : u32, 
    msg_type     : char,
    symbol       : String, 
    auction_type : char, 
    price        : u64, 
    shares       : u32
}

#[derive(Debug)]
pub struct AddOrderMsg {
    timestamp    : u32, 
    msg_type     : char,
    order_id     : u64, 
    side         : char, 
    shares       : u32, 
    symbol       : String,  
    price        : u64, 
    display      : char,
    part_id      : String  
}

#[derive(Debug)]
pub struct AuctionUpdateMsg {
    timestamp          : u32, 
    msg_type           : char,
    symbol             : String,
    auction_type       : char,
    reference_price    : u64,
    buyshares          : u32, 
    sellshares         : u32, 
    indicative_price   : u64, 
    auction_only_price : u64
}

fn from_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 36)
}

fn parse_opt_part_id( input : &str ) -> IResult<&str, String>
{
    if input.is_empty() {
        Ok(("", String::from("")))
    } else {
        let (first, last) = input.split_at(4);
        Ok((last, String::from(first)))
    }
}

named!(parse_auction_summary<&str, AuctionSummaryMsg>,  
    do_parse!(
        _1 : map_res!(take!(8),  FromStr::from_str) >>
        _2 : char!('J')                             >>
        _3 : map_res!(take!(8),  FromStr::from_str) >>
        _4 : map_res!(take!(1),  FromStr::from_str) >>
        _5 : map_res!(take!(10), FromStr::from_str) >>
        _6 : map_res!(take!(10), FromStr::from_str) >>
        (AuctionSummaryMsg{ timestamp    : _1, 
                            msg_type     : _2, 
                            symbol       : _3, 
                            auction_type : _4, 
                            price        : _5, 
                            shares       : _6 
                       } )  
    )
);

named!(parse_add_order<&str, AddOrderMsg>,  
    do_parse!(
        _1 : map_res!(take!(8),  FromStr::from_str) >>
        _2 : alt!(char!('A') | char!('d'))          >>
        _3 : map_res!(take!(12), from_hex)          >>
        _4 : map_res!(take!(1),  FromStr::from_str) >>
        _5 : map_res!(take!(6),  FromStr::from_str) >>
        _6 : map_res!(take!(6),  FromStr::from_str) >>
        _7 : map_res!(take!(10), FromStr::from_str) >>
        _8 : map_res!(take!(1),  FromStr::from_str) >>
        _9 : parse_opt_part_id                      >>
        (AddOrderMsg{ timestamp : _1, 
                      msg_type  : _2, 
                      order_id  : _3, 
                      side      : _4,
                      shares    : _5, 
                      symbol    : _6, 
                      price     : _7, 
                      display   : _8,
                      part_id   : _9
                    })  
    )
);

named!(parse_auction_update<&str, AuctionUpdateMsg>,  
    do_parse!(
        _1 : map_res!(take!(8),  FromStr::from_str)                  >>
        _2 : char!('I')                                              >>
        _3 : map_res!(take!(8), FromStr::from_str)                   >>
        _4 : alt!(char!('O') | char!('C') | char!('H') | char!('I')) >>
        _5 : map_res!(take!(10),  FromStr::from_str)                 >>
        _6 : map_res!(take!(10),  FromStr::from_str)                 >>
        _7 : map_res!(take!(10), FromStr::from_str)                  >>
        _8 : map_res!(take!(10),  FromStr::from_str)                 >>
        _9 : map_res!(take!(10), FromStr::from_str)                  >>
        (AuctionUpdateMsg{ timestamp     : _1, 
                      msg_type           : _2, 
                      symbol             : _3, 
                      auction_type       : _4,
                      reference_price    : _5, 
                      buyshares          : _6, 
                      sellshares         : _7, 
                      indicative_price   : _8,
                      auction_only_price : _9
                    })  
    )
);
