use std::{cell::RefCell, collections::HashMap};

use anyhow::{bail, Context, Result};

/// Generates API data for all functions.
pub fn api_functions() -> Vec<ApiFunc> {
    let keysets = parse_keysets(include_str!("../headers/keysets.h"));
    let keyset_field_map = keysets
        .into_iter()
        .map(|keyset| (keyset.name.clone(), keyset))
        .collect::<HashMap<_, _>>();

    let api_headers = [
        include_str!("../headers/autocmd.h.generated.h"),
        include_str!("../headers/buffer.h.generated.h"),
        include_str!("../headers/command.h.generated.h"),
        include_str!("../headers/deprecated.h.generated.h"),
        include_str!("../headers/extmark.h.generated.h"),
        include_str!("../headers/options.h.generated.h"),
        include_str!("../headers/tabpage.h.generated.h"),
        include_str!("../headers/ui.h.generated.h"),
        include_str!("../headers/vim.h.generated.h"),
        include_str!("../headers/vimscript.h.generated.h"),
        include_str!("../headers/wasm.h.generated.h"),
        include_str!("../headers/win_config.h.generated.h"),
        include_str!("../headers/window.h.generated.h"),
    ];
    api_headers
        .iter()
        .flat_map(|header| parse_funcs(header, &keyset_field_map))
        .collect()
}

/// Generates the information for all API Keysets.
///
/// Keysets are dictionaries with fixed keys. See `src/nvim/api/keysets.h`.
pub fn api_keysets() -> Vec<ApiKeyset> {
    parse_keysets(include_str!("../headers/keysets.h"))
}

/// Represents an API function.
#[derive(Debug)]
pub struct ApiFunc {
    /// The original function name.
    pub name: String,
    /// Arguments of the function.
    pub args: ApiFuncArgs,
    /// The return value.
    pub return_: ApiFuncReturn,
    /// Extra attributes of the function
    pub attrs: ApiFuncAttrs,
}

impl ApiFunc {
    /// Returns the normalized WIT name of the function.
    pub fn wit_name(&self) -> String {
        normalized_wit_identifier(&self.name)
    }
}

/// Represents the function's arguments.
#[derive(Debug)]
pub struct ApiFuncArgs {
    /// The arguments
    pub args: Vec<ApiArg>,
    /// If a channel ID is required.  
    /// If true, channel ID should be passed as the first argument when calling the native
    /// function.
    pub has_channel_id: bool,
    /// If an arena is required.
    /// If true, an arena should be passed as the argument just after all normal arguments when
    /// calling the native function. The returned value will be allocated in the arena.
    pub has_arena: bool,
}

/// Represents a single function argument.
#[derive(Debug)]
pub struct ApiArg {
    /// The original argument name.
    pub name: String,
    /// The type of the argument.
    pub type_: ApiType,
}

impl ApiArg {
    /// Returns the normalized WIT name of the argument.
    pub fn wit_name(&self) -> String {
        normalized_wit_identifier(&self.name)
    }
}

/// Represents the function's return value
#[derive(Debug)]
pub struct ApiFuncReturn {
    /// The return type. If it is `None`, the native function returns void.
    pub type_: Option<ApiType>,
    /// Whether the function can return an error.  
    /// If true, an `Error` struct should be passed as the last argument when calling the native
    /// function.
    pub has_error: bool,
}

/// Represents all possible API types.
#[derive(Debug)]
pub enum ApiType {
    Boolean,
    Integer,
    Float,
    String,
    Array(ApiArrayType),
    Dictionary(ApiDictionaryType),
    Keyset(ApiKeyset),
    LuaRef,
    Buffer,
    Window,
    Tabpage,
    Object,
}

/// Represents an Array type.
#[derive(Debug)]
pub struct ApiArrayType {
    /// The inner type of the array.
    ///
    /// Notice that it is only a *type hint*. API `Array` always store elements as an `Object.
    /// If `None`, the type is opaque. Arrays with an inner type is annotate with `ArrayOf(...)`,
    /// see `src/nvim/api/private/defs.h`.
    pub inner_type: Option<Box<ApiType>>,
}

/// Represents an Dictionary type.
#[derive(Debug)]
pub struct ApiDictionaryType {
    /// The type of the values of the dictionary.
    ///
    /// Notice that it is only a *type hint*. API `Array` always store elements as an `Object.
    /// If `None`, the type is opaque.
    pub inner_type: Option<Box<ApiType>>,
}

/// Represents an API keyset.
///
/// Keysets are dictionaries with fixed keys. See `src/nvim/api/keysets.h`. The fields are always
/// `Object`s.
#[derive(Debug, Clone)]
pub struct ApiKeyset {
    /// The original type name of the keyset.
    pub name: String,
    /// The fields of the keyset.
    pub fields: Vec<ApiField>,
}

impl ApiKeyset {
    /// Returns the normalized WIT type name of the keyset.
    pub fn wit_name(&self) -> String {
        normalized_wit_identifier(&format!("keyset-{}", self.name))
    }
}

/// Represents a field in a keyset.
#[derive(Debug, Clone)]
pub struct ApiField {
    /// The original name of the field.
    pub name: String,
}

impl ApiField {
    /// Returns the normalized WIT name of the field.
    pub fn wit_name(&self) -> String {
        normalized_wit_identifier(&self.name)
    }
}

/// Represents the extra attributes of the function.
#[derive(Default, Debug)]
pub struct ApiFuncAttrs {
    /// API level where the function was introduced.
    pub since: Option<ApiVersion>,
    /// True if this is an api-fast function. See `:help api-fast`.
    pub fast: bool,
    /// True if this is a remote only function.
    pub remote_only: bool,
    /// True if the function is not allowed when textlock is active. See `:textlock`.
    pub check_text_lock: bool,
}

/// Represents an API level.
#[derive(Debug)]
pub struct ApiVersion {
    pub version: i32,
}

struct TokenIterator {
    text: String,
    state: RefCell<TokenIteratorState>,
}

struct TokenIteratorState {
    len: usize,
    next_token_start: usize,
    next_token_end: Option<usize>,
}

impl TokenIteratorState {
    fn new(len: usize) -> Self {
        Self {
            len,
            next_token_start: 0,
            next_token_end: None,
        }
    }

    fn skip(&mut self, offset: usize) {
        self.next_token_start += offset;
    }

    fn set_peek_end(&mut self, offset: usize) {
        self.next_token_end = Some(self.next_token_start + offset);
    }

    fn is_peeked(&self) -> bool {
        self.next_token_end.is_some()
    }

    fn is_end(&self) -> bool {
        self.next_token_start == self.len
    }

    fn get_rest_str<'a>(&self, s: &'a str) -> &'a str {
        assert!(
            !self.is_peeked(),
            "It doesn't make sense to call `get_rest_str` when the peeked token has not been consumed"
        );
        &s[self.next_token_start..]
    }

    fn consume_next<'a>(&mut self, s: &'a str) -> &'a str {
        let start = self.next_token_start;
        let end = self
            .next_token_end
            .take()
            .expect("`peek_token_end` should not be None");
        self.next_token_start = end;

        &s[start..end]
    }

    fn get_peek_str<'a>(&self, s: &'a str) -> Option<&'a str> {
        Some(&s[self.next_token_start..self.next_token_end?])
    }
}

const SPECIAL_CHARS: [char; 5] = ['(', ')', ',', '*', ';'];
const DLLEXPORT: &str = "DLLEXPORT";

impl TokenIterator {
    fn new(text: String) -> Self {
        let len = text.len();
        Self {
            text,
            state: RefCell::new(TokenIteratorState::new(len)),
        }
    }

    fn peek_token(&self) -> Option<&str> {
        self.peek_token_inner()?;
        self.state.borrow().get_peek_str(&self.text)
    }

    fn peek_token_inner(&self) -> Option<()> {
        let mut state = self.state.borrow_mut();
        if state.is_end() {
            return None;
        }
        if state.is_peeked() {
            return Some(());
        }

        let mut rest_chars = state.get_rest_str(&self.text).chars();
        let mut c;

        // Skip the whitespaces
        loop {
            // We haven't reach the end here.
            c = rest_chars.next().unwrap();
            if !c.is_ascii_whitespace() {
                break;
            }
            state.skip(c.len_utf8());
        }

        // If the first character is a special character, return it.
        let mut offset = c.len_utf8();
        if SPECIAL_CHARS.contains(&c) {
            state.set_peek_end(offset);
            return Some(());
        }

        // Else, this is an identifier. Keep going until we find a special character.
        for c in rest_chars {
            if c.is_ascii_whitespace() || SPECIAL_CHARS.contains(&c) {
                break;
            }
            offset += c.len_utf8();
        }
        state.set_peek_end(offset);
        Some(())
    }

    fn next_token(&self) -> Option<&str> {
        self.peek_token_inner()?;
        Some(self.state.borrow_mut().consume_next(&self.text))
    }

    fn expect_next_token(&self, token: &str) -> Result<()> {
        match self.next_token() {
            None => bail!("Expect '{token}', got end of line"),
            Some(x) if x != token => bail!("Expect '{token}', got '{x}'"),
            _ => Ok(()),
        }
    }

    fn next_identifier(&self) -> Result<&str> {
        match self.next_token() {
            None => bail!("Expect an identifier, got end of line"),
            Some(name)
                if name.len() == 1 && SPECIAL_CHARS.contains(&name.chars().next().unwrap()) =>
            {
                bail!("Expect an identifier, got '{name}'")
            }
            Some(name) => Ok(name),
        }
    }

    fn next_number(&self) -> Result<i32> {
        let token = self
            .next_token()
            .context("Expect a number, got end of line")?;
        token
            .parse::<i32>()
            .context(format!("Expect a number, got '{token}'"))
    }
}

fn parse_single_arg(it: &TokenIterator) -> Result<&str> {
    it.expect_next_token("(")?;
    let result = it.next_identifier()?;
    it.expect_next_token(")")?;

    Ok(result)
}

fn parse_type(it: &TokenIterator) -> Result<ParseTypeCase> {
    let name = it.next_identifier()?;
    let type_ = match name {
        "void" => ParseTypeCase::Void,
        "uint64_t" => ParseTypeCase::ChannelId,
        "Error" => ParseTypeCase::Error,
        "Arena" => ParseTypeCase::Arena,
        "Dict" => {
            let keyset = parse_single_arg(it)?;
            ParseTypeCase::Keyset(keyset.to_owned())
        }
        tp if ApiType::str_is_nested_type(tp) => {
            let inner_type = parse_single_arg(it)?;
            ParseTypeCase::Normal(ApiType::nested_type_from_strs(name, inner_type).context(
                format!("Type `{inner_type}` is not a valid inner type for `{name}`"),
            )?)
        }
        tp => ParseTypeCase::Normal(
            ApiType::simple_type_from_str(name).context(format!("Unknown type `{tp}`"))?,
        ),
    };

    Ok(type_)
}

fn parse_arg(it: &TokenIterator) -> Result<ParseApiArg> {
    let type_ = parse_type(it)?;
    let name = match &type_ {
        ParseTypeCase::Void => "",
        t if t.arg_is_pointer() => {
            it.expect_next_token("*")?;
            it.next_identifier()?
        }
        _ => it.next_identifier()?,
    }
    .to_owned();

    Ok(ParseApiArg { type_, name })
}

fn parse_args(it: &TokenIterator) -> Result<Vec<ParseApiArg>> {
    it.expect_next_token("(")?;
    let mut args = Vec::new();
    loop {
        args.push(parse_arg(it)?);
        let token = it.next_token().context("Unexpected end of line")?;
        if token == ")" {
            return Ok(args);
        }
        if token != "," {
            bail!("Expect ',' or ')', got '{token}'");
        }
    }
}

fn parse_attributes(it: &TokenIterator) -> Result<ParseAttribute> {
    let attr = it.next_identifier()?;
    Ok(match attr {
        "FUNC_API_SINCE" => {
            it.expect_next_token("(")?;
            let version = it.next_number()?;
            it.expect_next_token(")")?;

            ParseAttribute::Since(version)
        }
        "FUNC_API_FAST" => ParseAttribute::Fast,
        "FUNC_API_REMOTE_ONLY" => ParseAttribute::RemoteOnly,
        "FUNC_API_CHECK_TEXTLOCK" => ParseAttribute::CheckTextLock,
        _ => bail!("Unknown attribute: {attr}"),
    })
}

fn parse_line(line: &str, keyset_field_map: &KeysetFieldMap) -> Result<ApiFunc> {
    let it = TokenIterator::new(line.to_owned());
    let first_token = it.peek_token().context("Unexpected end of line")?;
    if first_token == DLLEXPORT {
        let _ = it.next_token().unwrap();
    }
    let return_type = match parse_type(&it)? {
        ParseTypeCase::Normal(type_) => Some(type_),
        ParseTypeCase::Void => None,
        t => bail!("Unexpected return type: {t:?}"),
    };
    let name = it.next_identifier()?;
    let parsed_args = parse_args(&it)?;

    let mut attrs = ApiFuncAttrs::default();
    while it
        .peek_token()
        .context("Expect ';' or attributes, got end of line")?
        != ";"
    {
        let attr = parse_attributes(&it)?;
        match attr {
            ParseAttribute::Since(version) => {
                attrs.since = Some(ApiVersion { version });
            }
            ParseAttribute::Fast => attrs.fast = true,
            ParseAttribute::RemoteOnly => attrs.remote_only = true,
            ParseAttribute::CheckTextLock => attrs.check_text_lock = true,
        }
    }

    let mut args = ApiFuncArgs {
        args: vec![],
        has_channel_id: false,
        has_arena: false,
    };
    let mut return_ = ApiFuncReturn {
        type_: return_type,
        has_error: false,
    };
    let len = parsed_args.len();
    for arg in parsed_args {
        match arg.type_ {
            ParseTypeCase::Void => {
                if len > 1 {
                    bail!("If the argument is void, then it must be the only argument, but there are more");
                }
            }
            ParseTypeCase::ChannelId => {
                args.has_channel_id = true;
            }
            ParseTypeCase::Arena => {
                args.has_arena = true;
            }
            ParseTypeCase::Error => {
                return_.has_error = true;
            }
            ParseTypeCase::Normal(type_) => {
                args.args.push(ApiArg {
                    type_,
                    name: arg.name,
                });
            }
            ParseTypeCase::Keyset(keyset) => {
                let info = keyset_field_map
                    .get(&keyset)
                    .context("Unknown keyset type {keyset}")?;
                args.args.push(ApiArg {
                    name: arg.name,
                    type_: ApiType::Keyset(info.clone()),
                })
            }
        }
    }

    Ok(ApiFunc {
        name: name.to_owned(),
        args,
        return_,
        attrs,
    })
}

fn parse_keyset_field(it: &TokenIterator) -> Result<&str> {
    let type_ = parse_type(it)?;
    if !matches!(type_, ParseTypeCase::Normal(ApiType::Object)) {
        bail!("Expect the type of a keyset field to be an object, got {type_:?}");
    }
    let name = it.next_identifier()?;
    it.expect_next_token(";")?;

    Ok(name)
}

fn parse_keyset(it: &TokenIterator) -> Result<Option<ApiKeyset>> {
    loop {
        match it.next_token() {
            None => return Ok(None),
            Some(token) if token == "typedef" => break,
            _ => continue,
        }
    }
    it.expect_next_token("struct")?;
    it.expect_next_token("{")?;
    let mut keys = vec![];
    while it.peek_token().context("Unexpected end of line")? != "}" {
        keys.push(parse_keyset_field(it)?);
    }
    it.expect_next_token("}")?;

    match parse_type(it)? {
        ParseTypeCase::Keyset(name) => Ok(Some(ApiKeyset {
            name,
            fields: keys
                .into_iter()
                .map(|x| ApiField { name: x.to_owned() })
                .collect(),
        })),
        type_ => {
            bail!("Expect a keyset type (i.e., `Dict(...)`), got {type_:?}");
        }
    }
}

fn parse_keysets(header: &str) -> Vec<ApiKeyset> {
    let header = header
        .lines()
        .filter(|x| {
            let x = x.trim();
            !x.starts_with('#') && !x.is_empty()
        })
        .collect::<Vec<_>>()
        .join("\n");

    let it = TokenIterator::new(header);
    let mut result = vec![];
    while let Some(keyset) = parse_keyset(&it).expect("Failed to parse keysets") {
        result.push(keyset);
    }

    result
}

type KeysetFieldMap = HashMap<String, ApiKeyset>;

fn parse_funcs(header: &str, keyset_field_map: &KeysetFieldMap) -> Vec<ApiFunc> {
    header
        .lines()
        .filter_map(|line| parse_line(line, keyset_field_map).ok())
        .collect()
}

impl ApiType {
    fn simple_type_from_str(name: &str) -> Option<Self> {
        Some(match name {
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Float" => Self::Float,
            "String" => Self::String,
            "Array" => Self::Array(ApiArrayType { inner_type: None }),
            "Dictionary" => Self::Dictionary(ApiDictionaryType { inner_type: None }),
            "LuaRef" => Self::LuaRef,
            "Buffer" => Self::Buffer,
            "Window" => Self::Window,
            "Tabpage" => Self::Tabpage,
            "Object" => Self::Object,
            _ => return None,
        })
    }

    fn str_is_nested_type(name: &str) -> bool {
        const NESTED_TYPES: [&str; 2] = ["ArrayOf", "Dict"];
        NESTED_TYPES.contains(&name)
    }

    fn nested_type_from_strs(name: &str, inner_type: &str) -> Option<Self> {
        Some(match name {
            "ArrayOf" => Self::Array(ApiArrayType {
                inner_type: Some(Box::new(ApiType::simple_type_from_str(inner_type)?)),
            }),
            "DictionaryOf" => Self::Dictionary(ApiDictionaryType {
                inner_type: Some(Box::new(ApiType::simple_type_from_str(inner_type)?)),
            }),
            _ => return None,
        })
    }
}

#[derive(Debug)]
enum ParseTypeCase {
    Normal(ApiType),
    Keyset(String),
    Void,
    ChannelId,
    Arena,
    Error,
}

impl ParseTypeCase {
    fn arg_is_pointer(&self) -> bool {
        matches!(self, Self::Keyset(_) | Self::Arena | Self::Error)
    }
}

struct ParseApiArg {
    type_: ParseTypeCase,
    name: String,
}

enum ParseAttribute {
    Since(i32),
    Fast,
    RemoteOnly,
    CheckTextLock,
}

fn normalized_wit_identifier(s: &str) -> String {
    let s = s
        .replace('_', "-")
        .split('-')
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    s
}
