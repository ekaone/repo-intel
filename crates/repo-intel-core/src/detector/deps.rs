use serde_json::Value;
use crate::types::{Skill, SignalKind, SkillSource};

/// Layer 1 — dependency fingerprinting.
///
/// Parses `package.json` (npm) and `Cargo.toml` dependencies and maps them to
/// skills with confidence scores. This single layer gives ~70% accuracy.
///
/// Returns a list of detected skills (may contain duplicates — deduplication
/// happens in `mod.rs` via the confidence-merge step).
pub fn detect_from_deps(signal_files: &[crate::types::SignalFile]) -> Vec<Skill> {
    let mut skills = Vec::new();

    for signal in signal_files {
        match signal.kind {
            SignalKind::PackageJson => {
                skills.extend(from_package_json(&signal.content));
            }
            SignalKind::CargoToml => {
                skills.extend(from_cargo_toml(&signal.content));
            }
            _ => {}
        }
    }

    skills
}

// ── package.json ──────────────────────────────────────────────────────────────

fn from_package_json(content: &str) -> Vec<Skill> {
    let mut skills = Vec::new();

    let Ok(json) = serde_json::from_str::<Value>(content) else {
        return skills;
    };

    // Merge dependencies + devDependencies into one flat dep list
    let mut dep_names: Vec<String> = Vec::new();

    for section in &["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(deps) = json.get(section).and_then(|v| v.as_object()) {
            dep_names.extend(deps.keys().cloned());
        }
    }

    // Also extract project name + description for ProjectMeta (stored as special skills)
    // (Context builder picks these up by name.)
    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
        skills.push(Skill {
            name: format!("__project_name:{name}"),
            confidence: 1.0,
            source: SkillSource::PackageJson,
        });
    }
    if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
        if !desc.is_empty() {
            skills.push(Skill {
                name: format!("__project_desc:{desc}"),
                confidence: 1.0,
                source: SkillSource::PackageJson,
            });
        }
    }

    // Map deps → skills
    for dep in &dep_names {
        let dep = dep.as_str();
        let matched = npm_skill_rules(dep);
        skills.extend(matched);
    }

    skills
}

/// npm dependency → Skill mapping table.
/// Returns zero or more skills for a given dependency name.
fn npm_skill_rules(dep: &str) -> Vec<Skill> {
    let src = SkillSource::PackageJson;
    let mut out = Vec::new();

    // Helper closure
    let skill = |name: &str, confidence: f32| Skill {
        name: name.to_string(),
        confidence,
        source: src.clone(),
    };

    match dep {
        // ── Frontend frameworks ───────────────────────────────────────────────
        "react" | "react-dom" => {
            out.push(skill("React", 0.99));
            out.push(skill("Component Architecture", 0.90));
        }
        "vue" | "@vue/core" => {
            out.push(skill("Vue.js", 0.99));
            out.push(skill("Component Architecture", 0.90));
        }
        "svelte" => out.push(skill("Svelte", 0.99)),
        "@angular/core" => out.push(skill("Angular", 0.99)),

        // ── SSR / meta-frameworks ─────────────────────────────────────────────
        "next" => {
            out.push(skill("Next.js", 0.99));
            out.push(skill("SSR", 0.99));
            out.push(skill("React", 0.97));
        }
        "nuxt" | "nuxt3" => {
            out.push(skill("Nuxt.js", 0.99));
            out.push(skill("SSR", 0.99));
            out.push(skill("Vue.js", 0.97));
        }
        "@remix-run/react" | "@remix-run/node" => {
            out.push(skill("Remix", 0.99));
            out.push(skill("SSR", 0.99));
        }
        "astro" => out.push(skill("Astro", 0.99)),
        "gatsby" => {
            out.push(skill("Gatsby", 0.99));
            out.push(skill("SSR", 0.90));
        }

        // ── Styling ───────────────────────────────────────────────────────────
        "tailwindcss" => out.push(skill("Tailwind CSS", 0.99)),
        "styled-components" => out.push(skill("Styled Components", 0.99)),
        "@emotion/react" | "@emotion/styled" => out.push(skill("Emotion CSS", 0.99)),
        "sass" | "node-sass" => out.push(skill("Sass/SCSS", 0.97)),

        // ── State management ──────────────────────────────────────────────────
        "zustand" => out.push(skill("State Management (Zustand)", 0.95)),
        "jotai" => out.push(skill("State Management (Jotai)", 0.95)),
        "recoil" => out.push(skill("State Management (Recoil)", 0.95)),
        "redux" | "@reduxjs/toolkit" => out.push(skill("State Management (Redux)", 0.97)),
        "mobx" => out.push(skill("State Management (MobX)", 0.95)),
        "pinia" => out.push(skill("State Management (Pinia)", 0.95)),

        // ── Data fetching ─────────────────────────────────────────────────────
        "@tanstack/react-query" | "react-query" => {
            out.push(skill("Server State Management", 0.95));
            out.push(skill("Data Fetching", 0.95));
        }
        "swr" => out.push(skill("Data Fetching (SWR)", 0.95)),
        "axios" => out.push(skill("HTTP Client", 0.85)),

        // ── Database / ORM ────────────────────────────────────────────────────
        "prisma" | "@prisma/client" => {
            out.push(skill("Database ORM (Prisma)", 0.97));
            out.push(skill("Database", 0.97));
        }
        "drizzle-orm" => {
            out.push(skill("Database ORM (Drizzle)", 0.97));
            out.push(skill("Database", 0.97));
        }
        "typeorm" => {
            out.push(skill("Database ORM (TypeORM)", 0.97));
            out.push(skill("Database", 0.97));
        }
        "mongoose" => {
            out.push(skill("MongoDB (Mongoose)", 0.97));
            out.push(skill("Database", 0.97));
        }
        "pg" | "postgres" => out.push(skill("PostgreSQL", 0.90)),
        "mysql2" => out.push(skill("MySQL", 0.90)),
        "better-sqlite3" | "@libsql/client" => out.push(skill("SQLite", 0.90)),

        // ── Testing ───────────────────────────────────────────────────────────
        "vitest" => {
            out.push(skill("Testing (Vitest)", 0.98));
            out.push(skill("Testing", 0.98));
        }
        "jest" => {
            out.push(skill("Testing (Jest)", 0.98));
            out.push(skill("Testing", 0.98));
        }
        "@playwright/test" | "playwright" => {
            out.push(skill("E2E Testing (Playwright)", 0.97));
            out.push(skill("Testing", 0.90));
        }
        "cypress" => {
            out.push(skill("E2E Testing (Cypress)", 0.97));
            out.push(skill("Testing", 0.90));
        }
        "@testing-library/react" => out.push(skill("Component Testing", 0.95)),

        // ── API servers ───────────────────────────────────────────────────────
        "express" => out.push(skill("Node.js API Server (Express)", 0.98)),
        "fastify" => out.push(skill("Node.js API Server (Fastify)", 0.98)),
        "hono" => out.push(skill("Node.js API Server (Hono)", 0.98)),
        "koa" => out.push(skill("Node.js API Server (Koa)", 0.97)),
        "@nestjs/core" => {
            out.push(skill("NestJS", 0.99));
            out.push(skill("Node.js API Server", 0.99));
        }
        "elysia" => out.push(skill("Bun API Server (Elysia)", 0.98)),

        // ── GraphQL ───────────────────────────────────────────────────────────
        "graphql" => out.push(skill("GraphQL", 0.97)),
        "@apollo/client" => {
            out.push(skill("GraphQL", 0.97));
            out.push(skill("Apollo Client", 0.97));
        }
        "@apollo/server" | "apollo-server" => {
            out.push(skill("GraphQL", 0.97));
            out.push(skill("Apollo Server", 0.97));
        }
        "graphql-yoga" => out.push(skill("GraphQL", 0.97)),

        // ── Realtime ──────────────────────────────────────────────────────────
        "socket.io" => out.push(skill("WebSockets / Realtime (Socket.io)", 0.90)),
        "ws" => out.push(skill("WebSockets", 0.85)),

        // ── Build tools ───────────────────────────────────────────────────────
        "vite" => out.push(skill("Vite", 0.95)),
        "webpack" => out.push(skill("Webpack", 0.90)),
        "turbo" => out.push(skill("Turborepo", 0.90)),
        "tsup" => out.push(skill("TypeScript Build (tsup)", 0.88)),

        // ── Runtime / infra ───────────────────────────────────────────────────
        "typescript" => out.push(skill("TypeScript", 0.99)),
        "zod" => out.push(skill("Schema Validation (Zod)", 0.92)),
        "trpc" | "@trpc/server" | "@trpc/client" => out.push(skill("tRPC", 0.97)),
        "stripe" => out.push(skill("Payments (Stripe)", 0.92)),
        "resend" | "nodemailer" | "@sendgrid/mail" => out.push(skill("Email Integration", 0.88)),

        _ => {} // Unknown dep — skip
    }

    out
}

// ── Cargo.toml ────────────────────────────────────────────────────────────────

fn from_cargo_toml(content: &str) -> Vec<Skill> {
    let mut skills = Vec::new();
    let src = SkillSource::CargoToml;

    let skill = |name: &str, confidence: f32| Skill {
        name: name.to_string(),
        confidence,
        source: src.clone(),
    };

    // Parse as a generic TOML Value so we don't need a dedicated struct
    let Ok(table) = content.parse::<toml::Value>() else {
        return skills;
    };

    // Extract [package] name + description
    if let Some(pkg) = table.get("package") {
        if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
            skills.push(Skill {
                name: format!("__project_name:{name}"),
                confidence: 1.0,
                source: src.clone(),
            });
        }
        if let Some(desc) = pkg.get("description").and_then(|v| v.as_str()) {
            if !desc.is_empty() {
                skills.push(Skill {
                    name: format!("__project_desc:{desc}"),
                    confidence: 1.0,
                    source: src.clone(),
                });
            }
        }
    }

    // Collect all dependency names from [dependencies] and [dev-dependencies]
    let mut dep_names: Vec<String> = Vec::new();
    for section in &["dependencies", "dev-dependencies"] {
        if let Some(deps) = table.get(section).and_then(|v| v.as_table()) {
            dep_names.extend(deps.keys().cloned());
        }
    }

    // Language baseline — if Cargo.toml exists, this is a Rust project
    skills.push(skill("Rust", 0.99));

    for dep in &dep_names {
        let matched = cargo_skill_rules(dep.as_str(), &src);
        skills.extend(matched);
    }

    skills
}

fn cargo_skill_rules(dep: &str, src: &SkillSource) -> Vec<Skill> {
    let skill = |name: &str, confidence: f32| Skill {
        name: name.to_string(),
        confidence,
        source: src.clone(),
    };

    match dep {
        // ── Web frameworks ────────────────────────────────────────────────────
        "axum" => vec![
            skill("Rust Web Server (Axum)", 0.99),
            skill("Async Rust", 0.95),
        ],
        "actix-web" => vec![
            skill("Rust Web Server (Actix)", 0.99),
            skill("Async Rust", 0.95),
        ],
        "warp" => vec![skill("Rust Web Server (Warp)", 0.97)],
        "rocket" => vec![skill("Rust Web Server (Rocket)", 0.97)],

        // ── Database ──────────────────────────────────────────────────────────
        "sqlx" => vec![
            skill("Rust Database (SQLx)", 0.97),
            skill("Database", 0.97),
        ],
        "diesel" => vec![
            skill("Rust Database (Diesel)", 0.97),
            skill("Database", 0.97),
        ],
        "sea-orm" => vec![
            skill("Rust Database (SeaORM)", 0.97),
            skill("Database", 0.97),
        ],

        // ── Async runtime ─────────────────────────────────────────────────────
        "tokio" => vec![skill("Async Rust (Tokio)", 0.99)],
        "async-std" => vec![skill("Async Rust (async-std)", 0.97)],

        // ── Serialization ─────────────────────────────────────────────────────
        "serde" | "serde_json" => vec![skill("Serde / JSON Serialization", 0.92)],

        // ── CLI ───────────────────────────────────────────────────────────────
        "clap" => vec![skill("Rust CLI (Clap)", 0.92)],

        // ── WASM ──────────────────────────────────────────────────────────────
        "wasm-bindgen" | "wasm-pack" => vec![skill("WebAssembly (Rust)", 0.95)],

        // ── gRPC ─────────────────────────────────────────────────────────────
        "tonic" => vec![skill("gRPC (Tonic)", 0.95)],

        _ => vec![],
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SignalFile, SignalKind};
    use std::path::PathBuf;

    fn pkg_signal(content: &str) -> SignalFile {
        SignalFile {
            kind: SignalKind::PackageJson,
            path: PathBuf::from("package.json"),
            content: content.to_string(),
        }
    }

    fn cargo_signal(content: &str) -> SignalFile {
        SignalFile {
            kind: SignalKind::CargoToml,
            path: PathBuf::from("Cargo.toml"),
            content: content.to_string(),
        }
    }

    fn has_skill(skills: &[Skill], name: &str) -> bool {
        skills.iter().any(|s| s.name.contains(name))
    }

    #[test]
    fn detects_react_and_nextjs() {
        let signal = pkg_signal(r#"{
            "dependencies": { "next": "14", "react": "18", "react-dom": "18" }
        }"#);
        let skills = detect_from_deps(&[signal]);
        assert!(has_skill(&skills, "Next.js"));
        assert!(has_skill(&skills, "React"));
        assert!(has_skill(&skills, "SSR"));
    }

    #[test]
    fn detects_tailwind_and_vitest() {
        let signal = pkg_signal(r#"{
            "devDependencies": { "tailwindcss": "3", "vitest": "1" }
        }"#);
        let skills = detect_from_deps(&[signal]);
        assert!(has_skill(&skills, "Tailwind CSS"));
        assert!(has_skill(&skills, "Vitest"));
    }

    #[test]
    fn detects_prisma_as_database() {
        let signal = pkg_signal(r#"{
            "dependencies": { "prisma": "5", "@prisma/client": "5" }
        }"#);
        let skills = detect_from_deps(&[signal]);
        assert!(has_skill(&skills, "Prisma"));
        assert!(has_skill(&skills, "Database"));
    }

    #[test]
    fn detects_rust_axum_from_cargo() {
        let signal = cargo_signal(r#"
            [package]
            name = "my-api"
            version = "0.1.0"

            [dependencies]
            axum = "0.7"
            sqlx = { version = "0.7", features = ["postgres"] }
            tokio = { version = "1", features = ["full"] }
        "#);
        let skills = detect_from_deps(&[signal]);
        assert!(has_skill(&skills, "Rust"));
        assert!(has_skill(&skills, "Axum"));
        assert!(has_skill(&skills, "SQLx"));
        assert!(has_skill(&skills, "Tokio"));
    }

    #[test]
    fn extracts_project_name_from_package_json() {
        let signal = pkg_signal(r#"{ "name": "my-awesome-app" }"#);
        let skills = detect_from_deps(&[signal]);
        assert!(has_skill(&skills, "__project_name:my-awesome-app"));
    }

    #[test]
    fn ignores_malformed_json_gracefully() {
        let signal = pkg_signal("not valid json {{{");
        let skills = detect_from_deps(&[signal]);
        assert!(skills.is_empty());
    }
}