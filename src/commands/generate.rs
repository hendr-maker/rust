//! Generate command - Code generation for project components.

use crate::cli::args::{GenerateArgs, GenerateComponent};
use crate::errors::AppResult;
use crate::utils::templates;

/// Execute the generate command
pub async fn execute(args: GenerateArgs) -> AppResult<()> {
    match args.component {
        GenerateComponent::Entity { name } => {
            tracing::info!("Generating entity: {}", name);
            templates::generate_entity(&name)?;
            println!("Created: src/domain/{}.rs", name);
            println!("Created: src/infra/repositories/{}_repository.rs", name);
            println!("Don't forget to update mod.rs files!");
        }
        GenerateComponent::Migration { name } => {
            tracing::info!("Generating migration: {}", name);
            templates::generate_migration(&name)?;
            println!("Created migration: {}", name);
        }
        GenerateComponent::Service { name } => {
            tracing::info!("Generating service: {}", name);
            templates::generate_service(&name)?;
            println!("Created: src/services/{}_service.rs", name);
            println!("Don't forget to update mod.rs and wire in main.rs!");
        }
    }

    Ok(())
}
