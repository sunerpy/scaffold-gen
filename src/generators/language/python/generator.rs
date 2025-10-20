use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

// use crate::project::ProjectParams;
use crate::generators::core::{
    Generator, Parameters, ProjectGenerator as ProjectGeneratorTrait, TemplateProcessor,
};
