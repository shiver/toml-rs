// extern crate toml;
//
// use toml::Value;
//
// #[test]
// fn lookup_mut_change() {
//     let toml = r#"
//           [test]
//           foo = "bar"
//
//           [[values]]
//           foo = "baz"
//
//           [[values]]
//           foo = "qux"
//     "#;
//
//     let mut value: Value = toml.parse().unwrap();
//     {
//       let foo = &mut value["values"][0]["foo"];
//       *foo = Value::String(String::from("bar"));
//     }
//     let foo = &value["values"][0]["foo"];
//     assert_eq!(foo.as_str().unwrap(), "bar");
// }

// #[test]
// fn lookup_mut_valid() {
//     let toml = r#"
//           [test]
//           foo = "bar"
//
//           [[values]]
//           foo = "baz"
//
//           [[values]]
//           foo = "qux"
//     "#;
//
//     let mut value: Value = toml.parse().unwrap();
//
//     {
//         let test_foo = value.lookup_mut("test.foo").unwrap();
//         assert_eq!(test_foo.as_str().unwrap(), "bar");
//     }
//
//     {
//         let foo1 = value.lookup_mut("values.1.foo").unwrap();
//         assert_eq!(foo1.as_str().unwrap(), "qux");
//     }
//
//     assert!(value.lookup_mut("test.bar").is_none());
//     assert!(value.lookup_mut("test.foo.bar").is_none());
// }
//
// #[test]
// fn lookup_mut_invalid_index() {
//     let toml = r#"
//         [[values]]
//         foo = "baz"
//     "#;
//
//     let mut value: Value = toml.parse().unwrap();
//
//     {
//         let foo = value.lookup_mut("test.foo");
//         assert!(foo.is_none());
//     }
//
//     {
//         let foo = value.lookup_mut("values.100.foo");
//         assert!(foo.is_none());
//     }
//
//     {
//         let foo = value.lookup_mut("values.str.foo");
//         assert!(foo.is_none());
//     }
// }
//
// #[test]
// fn lookup_mut_self() {
//     let mut value: Value = r#"foo = "bar""#.parse().unwrap();
//
//     {
//         let foo = value.lookup_mut("foo").unwrap();
//         assert_eq!(foo.as_str().unwrap(), "bar");
//     }
//
//     let foo = value.lookup_mut("").unwrap();
//     assert!(foo.as_table().is_some());
//
//     let baz = foo.lookup_mut("foo").unwrap();
//     assert_eq!(baz.as_str().unwrap(), "bar");
// }
//
// #[test]
// fn lookup_valid() {
//     let toml = r#"
//           [test]
//           foo = "bar"
//
//           [[values]]
//           foo = "baz"
//
//           [[values]]
//           foo = "qux"
//     "#;
//
//     let value: Value = toml.parse().unwrap();
//
//     let test_foo = value.lookup("test.foo").unwrap();
//     assert_eq!(test_foo.as_str().unwrap(), "bar");
//
//     let foo1 = value.lookup("values.1.foo").unwrap();
//     assert_eq!(foo1.as_str().unwrap(), "qux");
//
//     assert!(value.lookup("test.bar").is_none());
//     assert!(value.lookup("test.foo.bar").is_none());
// }
//
// #[test]
// fn lookup_invalid_index() {
//     let toml = r#"
//         [[values]]
//         foo = "baz"
//     "#;
//
//     let value: Value = toml.parse().unwrap();
//
//     let foo = value.lookup("test.foo");
//     assert!(foo.is_none());
//
//     let foo = value.lookup("values.100.foo");
//     assert!(foo.is_none());
//
//     let foo = value.lookup("values.str.foo");
//     assert!(foo.is_none());
// }
//
// #[test]
// fn lookup_self() {
//     let value: Value = r#"foo = "bar""#.parse().unwrap();
//
//     let foo = value.lookup("foo").unwrap();
//     assert_eq!(foo.as_str().unwrap(), "bar");
//
//     let foo = value.lookup("").unwrap();
//     assert!(foo.as_table().is_some());
//
//     let baz = foo.lookup("foo").unwrap();
//     assert_eq!(baz.as_str().unwrap(), "bar");
// }
//
// #[test]
// fn lookup_advanced() {
//     let value: Value = "[table]\n\"value\" = 0".parse().unwrap();
//     let looked = value.lookup("table.\"value\"").unwrap();
//     assert_eq!(*looked, Value::Integer(0));
// }
//
// #[test]
// fn lookup_advanced_table() {
//     let value: Value = "[table.\"name.other\"]\nvalue = \"my value\"".parse().unwrap();
//     let looked = value.lookup(r#"table."name.other".value"#).unwrap();
//     assert_eq!(*looked, Value::String(String::from("my value")));
// }
//
// #[test]
// fn lookup_mut_advanced() {
//     let mut value: Value = "[table]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     let looked = value.lookup_mut("table.\"value\".1").unwrap();
//     assert_eq!(*looked, Value::Integer(1));
// }
//
// #[test]
// fn single_dot() {
//     let value: Value = "[table]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     assert_eq!(None, value.lookup("."));
// }
//
// #[test]
// fn array_dot() {
//     let value: Value = "[table]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     assert_eq!(None, value.lookup("0."));
// }
//
// #[test]
// fn dot_inside() {
//     let value: Value = "[table]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     assert_eq!(None, value.lookup("table.\"value.0\""));
// }
//
// #[test]
// fn table_with_quotes() {
//     let value: Value = "[table.\"element\"]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     assert_eq!(None, value.lookup("\"table.element\".\"value\".0"));
// }
//
// #[test]
// fn table_with_quotes_2() {
//     let value: Value = "[table.\"element\"]\n\"value\" = [0, 1, 2]".parse().unwrap();
//     assert_eq!(Value::Integer(0), *value.lookup("table.\"element\".\"value\".0").unwrap());
// }
//
// #[test]
// fn control_characters() {
//     let value = Value::String("\x05".to_string());
//     assert_eq!(value.to_string(), r#""\u0005""#);
// }
