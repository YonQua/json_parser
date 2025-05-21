use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char as char_parser, multispace0, none_of},
    combinator::{map, value},
    multi::{many0, separated_list0},
    number::complete::recognize_float,
    sequence::{delimited, separated_pair},
};
use serde_json::Value;

fn main() {
    let input = r#"
    {
        "name": "John\nDoe",
        "age": 30,
        "is_student": false,
        "courses": [
            "Math",
            "Science",
            "History"
        ],
        "address": {
            "street": "123 Main St",
            "city": "New York",
            "state": "NY"
        },
        "grades": {
            "Math": 90,
            "Science": 85,
            "History": 88
        },
        "is_active": true,
        "balance": 1000.50,
        "null_value": null,
        "escaped_string": "This is a string with a newline\\n and a tab\\t character.",
        "empty_array": [],
        "empty_object": {},
        "nested_array": [
            [1, 2, 3],
            [4, 5, 6]
        ],
        "nested_object": {
            "key1": {
                "subkey1": "value1",
                "subkey2": "value2"
            },
            "key2": {
                "subkey3": "value3",
                "subkey4": "value4"
            }
        },
        "complex_value": {
            "array": [
                {"key1": "value1"},
                {"key2": "value2"}
            ],
            "object": {
                "key3": "value3",
                "key4": "value4"
            }
        }
    }
    "#;
    
    let result = parse_primary(input);
    match result {
        Ok((remaining, value)) => {
            if !remaining.trim().is_empty() {
                println!("警告：存在未解析的输入：{:?}", remaining);
            }
            println!("成功解析 JSON：{:#?}", value);
        }
        Err(e) => {
            println!("解析 JSON 时出错：{:?}", e);
        }
    }
}

/// 解析 null 值
/// 
/// 这个函数用于解析 JSON 中的 null 值。
/// 
/// 详细解释：
/// 1. value 函数是 nom 提供的一个解析器组合子，它有两个参数：
///    - 第一个参数是解析成功时要返回的值（在这里是 Value::Null）
///    - 第二个参数是实际的解析器（在这里是用于匹配 "null" 字符串的解析器）
/// 
/// 2. delimited 函数用于处理被其他内容包围的值，它有三个参数：
///    - 第一个参数 multispace0：匹配零个或多个空白字符（空格、换行等）
///    - 第二个参数 tag("null")：匹配字符串 "null"
///    - 第三个参数 multispace0：再次匹配零个或多个空白字符
/// 
/// 3. parse(input) 是最终执行解析的方法
/// 
/// 举例：
/// - 输入 "  null  " -> 成功，返回 Value::Null
/// - 输入 "null" -> 成功，返回 Value::Null
/// - 输入 "nul" -> 失败，不是完整的 "null"
/// - 输入 "NULL" -> 失败，大小写不匹配
/// 
/// 返回值：
/// - 成功时返回 Ok((剩余输入, Value::Null))
/// - 失败时返回 Err(错误信息)
fn parse_null(input: &str) -> IResult<&str, Value> {
    value( // value 函数的作用是：当解析成功时，返回指定的值
        Value::Null,  // 第一个参数：指定解析成功时要返回的值
        delimited(
            multispace0,     // 第一个 delimited 参数：匹配前导空白
            tag("null"),     // 第二个 delimited 参数：匹配 "null" 字符串
            multispace0      // 第三个 delimited 参数：匹配尾随空白
        ),  // delimited 的作用是处理被空白字符包围的 "null" 字符串
    )  // value 的作用是：当 delimited 解析成功时，返回 Value::Null
    .parse(input)  // 对输入字符串执行解析操作
}

/// 解析布尔值（true 或 false）
fn parse_bool(input: &str) -> IResult<&str, Value> {
    alt((  // 使用 alt 组合器选择两个解析器之一
        value(
            Value::Bool(true),
            delimited(multispace0, tag("true"), multispace0),  // 匹配 "true"
        ),
        value(
            Value::Bool(false),
            delimited(multispace0, tag("false"), multispace0),  // 匹配 "false"
        ),
    ))
    .parse(input)
}

/// 解析 JSON 中的数字值
/// 
/// 这个函数用于解析 JSON 中的数字（整数或浮点数）。
/// 
/// 详细解释：
/// 1. map 函数的作用：
///    - 它接收两个参数：一个解析器和一个转换函数
///    - 当解析器成功时，使用转换函数处理解析结果
///    - 相当于 Python 中的：result = conversion_function(parser_result)
/// 
/// 2. delimited 的处理过程：
///    - multispace0：匹配前面的空白字符（比如：" 123" 中的空格）
///    - recognize_float：匹配数字字符串（比如："123.45"）
///    - multispace0：匹配后面的空白字符（比如："123 " 中的空格）
/// 
/// 3. 字符串到数字的转换过程：
///    - 首先将字符串解析为 f64 类型的浮点数
///    - 然后将 f64 转换为 serde_json::Number 类型
///    - 最后包装为 JSON Value 类型
/// 
/// 举例：
/// - 输入：" 123.45 " 
///   1) 去掉前后空格
///   2) 识别 "123.45" 为数字
///   3) 转换为 JSON 数字值
/// 
/// 错误处理：
/// - 如果输入不是有效的数字格式，将返回错误
/// - 如果数字无法转换为 JSON 数字类型，将 panic
fn parse_number(input: &str) -> IResult<&str, Value> {
    map( // map 函数的作用是将解析结果转换为 JSON 数字
        // 第一步：处理输入字符串
        delimited(
            multispace0,      // 1.1: 匹配前导空白（例如："  123" 中的空格）
            recognize_float,   // 1.2: 识别浮点数字符串（例如："-123.45" 或 "42"）
            multispace0       // 1.3: 匹配尾随空白（例如："123  " 中的空格）
        ),
        // 第二步：转换函数，将字符串转为 JSON 数字
        |s: &str| {
            // 2.1: 将字符串解析为 f64 类型的浮点数
            // 例如："123.45" -> 123.45
            let num = s.parse::<f64>().unwrap();  

            // 2.2: 将 f64 转换为 serde_json 的 Number 类型
            // 这一步确保数字符合 JSON 标准
            // 例如：123.45 -> serde_json::Number
            Value::Number(serde_json::Number::from_f64(num).unwrap())
        },
    )
    .parse(input)  // 第三步：执行解析操作
}

/// 解析转义字符
/// 处理 JSON 字符串中的特殊字符，如 \n, \t 等
fn parse_escaped_char(input: &str) -> IResult<&str, char> {
    let (input, _) = char_parser('\\')(input)?;  // 首先匹配反斜杠
    alt((  // 然后匹配以下转义字符之一
        value('\"', char_parser('\"')),  // 引号
        value('\\', char_parser('\\')),  // 反斜杠
        value('/', char_parser('/')),    // 斜杠
        value('\n', char_parser('n')),   // 换行
        value('\r', char_parser('r')),   // 回车
        value('\t', char_parser('t')),   // 制表符
        value('\u{0008}', char_parser('b')),  // 退格
        value('\u{000C}', char_parser('f')),  // 换页
    )).parse(input)
}

/// 解析字符串
/// 处理普通字符和转义字符
fn parse_string(input: &str) -> IResult<&str, Value> {
    delimited( // 处理被引号包围的字符串
        char_parser('"'),  // 开始引号
        map( // 将解析结果转换为 JSON 字符串
            many0(alt((  // 匹配多个字符
                map(parse_escaped_char, |c| c),  // 处理转义字符
                none_of("\"\\"),  // 处理普通字符（非引号和反斜杠）
            ))),
            |chars| Value::String(chars.into_iter().collect())  // 将字符集合转换为字符串
        ),
        char_parser('"')  // 结束引号
    ).parse(input)
}

/// 解析数组
/// 处理由方括号包围的值列表
fn parse_array(input: &str) -> IResult<&str, Value> {
    delimited(
        delimited(multispace0, char_parser('['), multispace0),  // 开始方括号
        map(
            separated_list0(  // 解析由逗号分隔的值列表
                delimited(multispace0, char_parser(','), multispace0),
                parse_primary
            ),
            Value::Array  // 将值列表转换为 JSON 数组
        ),
        delimited(multispace0, char_parser(']'), multispace0)  // 结束方括号
    ).parse(input)
}

/// 解析对象
/// 处理由大括号包围的键值对列表
fn parse_object(input: &str) -> IResult<&str, Value> {
    delimited(
        delimited(multispace0, char_parser('{'), multispace0),  // 开始大括号
        map(
            separated_list0(  // 解析由逗号分隔的键值对列表
                delimited(multispace0, char_parser(','), multispace0),
                separated_pair(  // 解析键值对
                    delimited(multispace0, parse_string, multispace0),  // 键（必须是字符串）
                    char_parser(':'),  // 冒号分隔符
                    parse_primary  // 值（可以是任何 JSON 值）
                )
            ),
            |pairs| {  // 将键值对列表转换为 JSON 对象
                let mut map = serde_json::Map::new();
                for (key, value) in pairs {
                    if let Value::String(k) = key {
                        map.insert(k, value);
                    }
                }
                Value::Object(map)
            }
        ),
        delimited(multispace0, char_parser('}'), multispace0)  // 结束大括号
    ).parse(input)
}

/// 主解析函数
/// 可以解析任何类型的 JSON 值
fn parse_primary(input: &str) -> IResult<&str, Value> {
    delimited(
        multispace0,  // 前导空白
        alt((  // 尝试以下解析器之一
            parse_null,    // null 值
            parse_bool,    // 布尔值
            parse_number,  // 数字
            parse_string,  // 字符串
            parse_array,   // 数组
            parse_object,  // 对象
        )),
        multispace0   // 尾随空白
    ).parse(input)
}


