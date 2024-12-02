//!
//! The `solc --standard-json` output source.
//!

use std::collections::BTreeMap;

use boolinator::Boolinator;

use crate::standard_json::input::settings::error_type::ErrorType as StandardJsonInputSettingsErrorType;
use crate::standard_json::input::settings::warning_type::WarningType as StandardJsonInputSettingsWarningType;
use crate::standard_json::input::source::Source as StandardJSONInputSource;
use crate::standard_json::output::error::Error as StandardJsonOutputError;
use crate::version::Version;

///
/// The `solc --standard-json` output source.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// The source code ID.
    pub id: usize,
    /// The source code AST.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ast: Option<serde_json::Value>,
}

impl Source {
    ///
    /// Initializes a standard JSON source.
    ///
    /// Is used for projects compiled without `solc`.
    ///
    pub fn new(id: usize) -> Self {
        Self { id, ast: None }
    }

    ///
    /// Checks the AST node for the usage of `<address payable>`'s `send` and `transfer` methods.
    ///
    pub fn check_send_and_transfer(
        solc_version: &Version,
        ast: &serde_json::Value,
        id_paths: &BTreeMap<usize, &String>,
        sources: &BTreeMap<String, StandardJSONInputSource>,
    ) -> Option<StandardJsonOutputError> {
        let ast = ast.as_object()?;

        (ast.get("nodeType")?.as_str()? == "FunctionCall").as_option()?;

        let expression = ast.get("expression")?.as_object()?;
        (expression.get("nodeType")?.as_str()? == "MemberAccess").as_option()?;
        let member_name = expression.get("memberName")?.as_str()?;
        ["send", "transfer"].contains(&member_name).as_option()?;

        let expression = expression.get("expression")?.as_object()?;
        let type_descriptions = expression.get("typeDescriptions")?.as_object()?;
        let type_identifier = type_descriptions.get("typeIdentifier")?.as_str()?;
        let mut affected_types = vec!["t_address_payable"];
        if solc_version.default < semver::Version::new(0, 5, 0) {
            affected_types.push("t_address");
        }
        affected_types.contains(&type_identifier).as_option()?;

        Some(StandardJsonOutputError::error_send_and_transfer(
            ast.get("src")?.as_str(),
            id_paths,
            sources,
        ))
    }

    ///
    /// Checks the AST node for the usage of runtime code.
    ///
    pub fn check_runtime_code(
        ast: &serde_json::Value,
        id_paths: &BTreeMap<usize, &String>,
        sources: &BTreeMap<String, StandardJSONInputSource>,
    ) -> Option<StandardJsonOutputError> {
        let ast = ast.as_object()?;

        (ast.get("nodeType")?.as_str()? == "MemberAccess").as_option()?;
        (ast.get("memberName")?.as_str()? == "runtimeCode").as_option()?;

        let expression = ast.get("expression")?.as_object()?;
        let type_descriptions = expression.get("typeDescriptions")?.as_object()?;
        type_descriptions
            .get("typeIdentifier")?
            .as_str()?
            .starts_with("t_magic_meta_type")
            .as_option()?;

        Some(StandardJsonOutputError::error_runtime_code(
            ast.get("src")?.as_str(),
            id_paths,
            sources,
        ))
    }

    ///
    /// Checks the AST node for the `tx.origin` value usage.
    ///
    pub fn check_tx_origin(
        ast: &serde_json::Value,
        id_paths: &BTreeMap<usize, &String>,
        sources: &BTreeMap<String, StandardJSONInputSource>,
    ) -> Option<StandardJsonOutputError> {
        let ast = ast.as_object()?;

        (ast.get("nodeType")?.as_str()? == "MemberAccess").as_option()?;
        (ast.get("memberName")?.as_str()? == "origin").as_option()?;

        let expression = ast.get("expression")?.as_object()?;
        (expression.get("nodeType")?.as_str()? == "Identifier").as_option()?;
        (expression.get("name")?.as_str()? == "tx").as_option()?;

        Some(StandardJsonOutputError::warning_tx_origin(
            ast.get("src")?.as_str(),
            id_paths,
            sources,
        ))
    }

    ///
    /// Checks the AST node for the `origin` assembly instruction usage.
    ///
    pub fn check_assembly_origin(
        solc_version: &Version,
        ast: &serde_json::Value,
        id_paths: &BTreeMap<usize, &String>,
        sources: &BTreeMap<String, StandardJSONInputSource>,
    ) -> Option<StandardJsonOutputError> {
        let ast = ast.as_object()?;

        match ast.get("nodeType")?.as_str()? {
            "InlineAssembly" if solc_version.default < semver::Version::new(0, 6, 0) => {
                ast.get("operations")?
                    .as_str()?
                    .contains("origin()")
                    .as_option()?;
            }
            "YulFunctionCall" if solc_version.default >= semver::Version::new(0, 6, 0) => {
                (ast.get("functionName")?
                    .as_object()?
                    .get("name")?
                    .as_str()?
                    == "origin")
                    .as_option()?;
            }
            _ => return None,
        }

        Some(StandardJsonOutputError::warning_tx_origin(
            ast.get("src")?.as_str(),
            id_paths,
            sources,
        ))
    }

    ///
    /// Returns the list of messages for some specific parts of the AST.
    ///
    pub fn get_messages(
        ast: &serde_json::Value,
        id_paths: &BTreeMap<usize, &String>,
        sources: &BTreeMap<String, StandardJSONInputSource>,
        solc_version: &Version,
        suppressed_errors: &[StandardJsonInputSettingsErrorType],
        suppressed_warnings: &[StandardJsonInputSettingsWarningType],
    ) -> Vec<StandardJsonOutputError> {
        let mut messages = Vec::new();
        if !suppressed_errors.contains(&StandardJsonInputSettingsErrorType::SendTransfer) {
            if let Some(message) =
                Self::check_send_and_transfer(solc_version, ast, id_paths, sources)
            {
                messages.push(message);
            }
        }
        if let Some(message) = Self::check_runtime_code(ast, id_paths, sources) {
            messages.push(message);
        }
        if !suppressed_warnings.contains(&StandardJsonInputSettingsWarningType::TxOrigin) {
            if let Some(message) = Self::check_assembly_origin(solc_version, ast, id_paths, sources)
            {
                messages.push(message);
            }
            if let Some(message) = Self::check_tx_origin(ast, id_paths, sources) {
                messages.push(message);
            }
        }

        match ast {
            serde_json::Value::Array(array) => {
                for element in array.iter() {
                    messages.extend(Self::get_messages(
                        element,
                        id_paths,
                        sources,
                        solc_version,
                        suppressed_errors,
                        suppressed_warnings,
                    ));
                }
            }
            serde_json::Value::Object(object) => {
                for (_key, value) in object.iter() {
                    messages.extend(Self::get_messages(
                        value,
                        id_paths,
                        sources,
                        solc_version,
                        suppressed_errors,
                        suppressed_warnings,
                    ));
                }
            }
            _ => {}
        }

        messages
    }

    ///
    /// Returns the name of the last contract.
    ///
    pub fn last_contract_name(&self) -> anyhow::Result<String> {
        self.ast
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The AST is empty"))?
            .get("nodes")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("The last contract cannot be found in an empty list of nodes")
            })?
            .iter()
            .filter_map(
                |node| match node.get("nodeType").and_then(|node| node.as_str()) {
                    Some("ContractDefinition") => Some(node.get("name")?.as_str()?.to_owned()),
                    _ => None,
                },
            )
            .last()
            .ok_or_else(|| anyhow::anyhow!("The last contract not found in the AST"))
    }
}