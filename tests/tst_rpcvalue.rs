use chainpack::{RpcValue, Decimal};
use std::mem::size_of;

/// Setup function that is only run once, even if called multiple times.
fn init_log() {
    let _ = env_logger::builder().is_test(true).try_init();
    // INIT.call_once(|| {
    //     env_logger::init();
    // });
}

fn from_chainpack(data: &[u8]) -> RpcValue {
    return RpcValue::from_chainpack(data).unwrap()
}

fn to_chainpack(rv: &RpcValue) -> Vec<u8> {
    return rv.to_chainpack();
}

fn from_cpon(data: &str) -> RpcValue {
    return RpcValue::from_cpon(data).unwrap()
}

fn to_cpon(rv: &RpcValue) -> String {
    return rv.to_cpon();
}

#[test]
fn test_cpon_chainpack() {
    init_log();
    log::info!("------------- NULL ");
    {
        let rv = RpcValue::new(());
        assert_eq!(to_cpon(&rv), "null");
        //assert_eq!(to_chainpack(&rv), [PackingSchema::Null as u8]);
    }
    log::info!("------------- BOOL ");
    for b in [true, false].iter() {
        let rv = RpcValue::new(*b);
        assert_eq!(to_cpon(&rv), (if *b == true { "true" } else { "false" }));
        //assert_eq!(to_chainpack(&rv), (if *b == true { [PackingSchema::TRUE as u8] } else { [PackingSchema::FALSE as u8] }));
    }
    log::info!("------------- tiny uint ");
    for n in 0..64_u8 {
        let rv = RpcValue::new(n as u64);
        assert_eq!(to_cpon(&rv), (n.to_string() + "u"));
        assert_eq!(to_chainpack(&rv), [n]);
    }
    log::info!("------------- uint ");
    for i in 0..size_of::<u64>() {
        for j in 0..3 {
            let n = (1_u64 << (i * 8 + j * 3 + 1)) + 1;
            let rv = RpcValue::new(n);
            let cpon = to_cpon(&rv);
            let cpk = to_chainpack(&rv);
            log::debug!("\t cpon: {}", cpon);
            let rv_cpon = from_cpon(&cpon);
            let rv_cpk = from_chainpack(&cpk);
            //logD(n, cpon, cpk);
            assert_eq!(n.to_string() + "u", cpon);
            assert_eq!(rv_cpon, rv_cpk);
            assert_eq!(rv_cpk.as_u64(), n);
        }
    }
    log::info!("------------- tiny int ");
    for n in 0 .. 64_i8 {
        let rv = RpcValue::new(n as i64);
        assert_eq!(to_cpon(&rv), n.to_string());
        assert_eq!(to_chainpack(&rv), [n as u8 + 64]);
    }
    log::info!("------------- int ");
    for sig in [-1, 1].iter() {
        for i in 0..size_of::<i64>() {
            for j in 0 .. 3 {
                let n = ((*sig * 1_i64) << (i*8 + j*2+2)) + 1;
                let rv = RpcValue::new(n);
                let cpon = to_cpon(&rv);
                let cpk = to_chainpack(&rv);
                log::debug!("\t n: {} cpon: {}", n, cpon);
                let rv_cpon = from_cpon(&cpon);
                let rv_cpk = from_chainpack(&cpk);
                //logD(n, cpon, cpk);
                assert_eq!(n.to_string(), cpon);
                assert_eq!(rv_cpon, rv_cpk);
                assert_eq!(rv_cpk.as_i64(), n);
            }
        }
    }
    log::info!("------------- decimal ");
    {
        let mant = -123456;
        let exp_min = 1;
        let exp_max = 16;
        for exp in exp_min ..= exp_max {
            let n = Decimal::new(mant, exp);
            //let (m,e) = n.decode();
            log::debug!("\t mant: {} exp: {}, n: {}", mant, exp, n.to_cpon_string());
            let rv = RpcValue::new(n.clone());
            let cpon = to_cpon(&rv);
            let cpk = to_chainpack(&rv);
            let rv_cpon = from_cpon(&cpon);
            let rv_cpk = from_chainpack(&cpk);
            assert_eq!(n.to_cpon_string(), cpon);
            assert_eq!(rv_cpk.as_decimal(), n);
            let d1 = rv_cpk.as_decimal().to_f64();
            let d2 = rv_cpon.as_decimal().to_f64();
            assert_eq!(d1, d2);
        }
    }
    log::info!("------------- double");
    {
        let n_max = 1000000.;
        let n_min = -1000000.;
        let step = (n_max - n_min) / 100.1;
        let mut n = n_min;
        while n < n_max {
            let rv = RpcValue::new(n);
            let _ = to_cpon(&rv);
            let cpk = to_chainpack(&rv);
            let rv_cpk = from_chainpack(&cpk);
            // log::debug!("\t n: {} cpon: {}", n, cpon);
            assert_eq!(rv_cpk.as_f64(), n);
            n += step;
        }
    }
    {
        let step = -1.23456789e-10;
        let mut n = -f64::MAX / 10.;
        while n != 0. {
            let rv = RpcValue::new(n);
            let _ = to_cpon(&rv);
            let cpk = to_chainpack(&rv);
            let rv_cpk = from_chainpack(&cpk);
            //log::debug!("\t n: {:e} cpon: {}", n, cpon);
            assert_eq!(rv_cpk.as_f64(), n);
            n *= step;
        }
    }
    log::info!("------------- DateTime ");
    {
        let cpons = [
            ["d\"2018-02-02T00:00:00.001Z\"", "d\"2018-02-02T00:00:00.001Z\""],
            ["d\"2018-02-02T01:00:00.001+01\"", "d\"2018-02-02T01:00:00.001+01\""],
            ["d\"2018-12-02T00:00:00Z\"", "d\"2018-12-02T00:00:00Z\""],
            ["d\"2041-03-04T00:00:00-1015\"", "d\"2041-03-04T00:00:00-1015\""],
            ["d\"2041-03-04T00:00:00.123-1015\"", "d\"2041-03-04T00:00:00.123-1015\""],
            ["d\"1970-01-01T00:00:00Z\"", "d\"1970-01-01T00:00:00Z\""],
            ["d\"2017-05-03T05:52:03Z\"", "d\"2017-05-03T05:52:03Z\""],
            ["d\"2017-05-03T15:52:03.923Z\"", "d\"2017-05-03T15:52:03.923Z\""],
            ["d\"2017-05-03T15:52:03.920Z\"", "d\"2017-05-03T15:52:03.920Z\""],
            ["d\"2017-05-03T15:52:03.900Z\"", "d\"2017-05-03T15:52:03.900Z\""],
            ["d\"2017-05-03T15:52:03.000-0130\"", "d\"2017-05-03T15:52:03-0130\""],
            ["d\"2017-05-03T15:52:03.923+00\"", "d\"2017-05-03T15:52:03.923Z\""],
        ];
        for cpon in &cpons {
            log::debug!("---> cpon: {}", cpon[0]);
            let rv1 = from_cpon(cpon[0]);
            log::debug!("\trv: {} epoch: {} offset: {}", &rv1, &rv1.as_datetime().epoch_msec(), &rv1.as_datetime().utc_offset());
            let cpk = rv1.to_chainpack();
            let rv2 = from_chainpack(&cpk);
            let cpon2 = rv2.to_cpon();
            log::debug!("\tcpon: {} -> {}", cpon[0], cpon2);
            assert_eq!(cpon2, cpon[1]);
        }
    }
    log::info!("------------- cstring ");
    {
        let cpons = [
            ["", "\"\""],
            ["hello", "\"hello\""],
            ["\t\r", "\"\\t\\r\""],
            ["\0", "\"\\0\""],
            ["1\t\r\n", "\"1\\t\\r\\n\""],
            ["escaped zero \\0 here \t\r\n", "\"escaped zero \\\\0 here \\t\\r\\n\""],
        ];
        for cpon in &cpons {
            log::debug!("---> cpon: {}", cpon[1]);
            let rv1 = from_cpon(cpon[1]);
            log::debug!("\t rv1: {}", rv1);
            let cpk = rv1.to_chainpack();
            let rv2 = from_chainpack(&cpk);
            let cpon2 = rv2.to_cpon();
            assert_eq!(cpon2, cpon[1]);
            let rv3 = from_cpon(&cpon2);
            assert_eq!(rv3.as_str(), cpon[0]);
        }
    }
}

#[test]
fn test_conversions()
{
    init_log();
    log::info!("testConversions ------------");
    for lst in [
        [r#"/*comment 1*/{ /*comment 2*/
		    "foo"/*comment "3"*/: "bar", //comment to end of line
		    "baz" : 1,
            /*
            multiline comment
            "baz" : 1,
            "baz" : 1, // single inside multi
            */
		}"#, "{\"baz\":1,\"foo\":\"bar\"}"],
        [&(u64::MAX.to_string() + "u"), ""],
        [&(u64::MIN.to_string() + "u"), ""],
        [&i64::MAX.to_string(), ""],
        [&(i64::MIN+1).to_string(), ""],
        [&(u32::MAX.to_string() + "u"), ""],
        [&(u32::MIN.to_string() + "u"), ""],
        [&i32::MAX.to_string(), ""],
        [&i32::MIN.to_string(), ""],
        ["true", ""],
        ["false", ""],
        ["\"\"", ""],
        ["1u", ""],
        ["134", ""],
        ["7", ""],
        ["-2", ""],
        ["0xab", "171"],
        ["-0xCD", "-205"],
        ["0x1a2b3c4d", "439041101"],
        ["223.", ""],
        ["2.30", ""],
        ["12.3e-10", "123e-11"],
        ["-0.00012", "-12e-5"],
        ["-1234567890.", "-1234567890."],
        ["\"foo\"", ""],
        ["[]", ""],
        ["[1]", ""],
        ["[1,]", "[1]"],
        ["[1,2,3]", ""],
        ["[[]]", ""],
        ["{\"foo\":\"bar\"}", ""],
        ["i{1:2}", ""],
        ["i{\n\t1: \"bar\",\n\t345 : \"foo\",\n}", "i{1:\"bar\",345:\"foo\"}"],
        ["[1u,{\"a\":1},2.30]", ""],
        ["<1:2>3", ""],
        ["[1,<7:8>9]", ""],
        ["<>1", "1"],
        ["<8:3u>i{2:[[\".broker\",<1:2>true]]}", ""],
        ["<1:2,\"foo\":\"bar\">i{1:<7:8>9}", ""],
        ["<1:2,\"foo\":<5:6>\"bar\">[1u,{\"a\":1},2.30]", ""],
        ["i{1:2 // comment to end of line\n}", "i{1:2}"],
        [r#"/*comment 1*/{ /*comment 2*/
		    "foo"/*comment "3"*/: "bar", //comment to end of line
		    "baz" : 1,
            /*
            multiline comment
            "baz" : 1,
            "baz" : 1, // single inside multi
            */
		}"#, "{\"baz\":1,\"foo\":\"bar\"}"],
        ["<1:2>[3,<4:5>6]", ""],
        ["<4:\"svete\">i{2:<4:\"svete\">[0,1]}", ""],
        ["d\"2019-05-03T11:30:00-0700\"", "d\"2019-05-03T11:30:00-07\""],
        ["d\"2018-02-02T00:00:00Z\"", ""],
        ["d\"2027-05-03T11:30:12.345+01\"", ""],
    ].iter() {
        let cpon1 = lst[0];
        let cpon2 = if lst[1].len() == 0 { lst[0] } else { lst[1] };

        log::debug!("-------------> cpon1: {}", cpon1);
        let rv1 = from_cpon(cpon1);
        //log::debug!("\t rv1: {}", &rv1);
        let cpk1 = rv1.to_chainpack();
        //log::debug!("\t cpk1: {:?}", &cpk1);
        let rv2 = from_chainpack(&cpk1);
        //log::debug!("\t rv2: {}", &rv2);
        let cpn2 = rv2.to_cpon();
//logD(cpon2, "\t--cpon------>\t", cpn2);
        assert_eq!(cpn2, cpon2);
    }
}
/*
void testDateTime()
{
// same points in time
RpcValue v1 = from_cpon("\d"2017-05-03T18:30:00Z\"");
RpcValue v2 = from_cpon("\d"2017-05-03T22:30:00+04\"");
RpcValue v3 = from_cpon("\d"2017-05-03T11:30:00-0700\"");
RpcValue v4 = from_cpon("\d"2017-05-03T15:00:00-0330\"");
assert_eq!(v1.datetime.msecsSinceEpoch == v2.datetime.msecsSinceEpoch);
assert_eq!(v2.datetime.msecsSinceEpoch == v3.datetime.msecsSinceEpoch);
assert_eq!(v3.datetime.msecsSinceEpoch == v4.datetime.msecsSinceEpoch);
assert_eq!(v4.datetime.msecsSinceEpoch == v1.datetime.msecsSinceEpoch);
}

int main(string[] args)
{
args = globalLog.setCLIOptions(args);
test_vals();
testConversions();
testDateTime();
logInfo("PASSED");
return 0;
}
*/
