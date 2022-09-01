#[cfg(test)]
mod tests {
    // #[macro_export]
    // macro_rules! format_vec {
    //     (
    //         $pattern:ident,
    //         $($arg:tt)*
    //     ) => {
    //      format!($($arg)*)
    //     };
    // }

    // fn parse_str(args: &Arguments) {
    //     if let Some(s) = args.as_str() {
    //         println!("{}", s)
    //     } else {
    //         println!("{}", &args.to_string());
    //     }
    // }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        let mut param = String::from("git commit -m \"{} {}\"");

        let rest = param.matches("{}");

        println!("Matches: {}", rest.count());

        let args = vec!["arg1", "arg2"];

        // let str_result = std::fmt::format(format_args!("{}", args.as_slice()));

        for arg in args {
            param = param.replacen("{}", &arg, 1);
        }

        println!("{:?}", &param);

        assert_eq!(result, 4);
    }
}