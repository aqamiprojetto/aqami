# AQAMI — AI-Native Solana Framework

## Overview

AQAMI is an AI-native framework for building Solana applications in Rust. Unlike traditional blockchain frameworks designed primarily for human developers, AQAMI is built from the ground up to be easily understood, generated, and maintained by AI coding agents such as ChatGPT, Codex, Claude, and future autonomous development systems.

The framework emphasizes explicit structure, predictable conventions, and machine-readable metadata to maximize code generation accuracy and reduce ambiguity.

## Goals

* Enable AI agents to generate high-quality Solana programs with minimal context.
* Reduce framework-specific complexity and hidden behaviors.
* Provide deterministic project structures and coding patterns.
* Improve maintainability, readability, and onboarding for both humans and AI.
* Support rapid development of decentralized applications on Solana.

## Core Principles

### AI-First Design

Every framework feature must be understandable from source code and metadata without requiring extensive documentation lookup.

### Explicit Over Implicit

No hidden magic, automatic discovery, or runtime behavior that is difficult for AI systems to infer.

### Convention-Driven Structure

Projects follow a standardized layout that allows AI agents to navigate and modify code safely.

### Machine-Readable Metadata

Programs, accounts, instructions, and events are described through structured metadata consumable by AI tools.

### Rust-Native Development

AQAMI embraces idiomatic Rust while minimizing unnecessary abstraction layers.

## Key Components

### Runtime Library

Core primitives for accounts, instructions, validation, events, and state management.

### CLI Tooling

Project creation, code generation, validation, testing, and deployment utilities.

### MCP Server

An official Model Context Protocol (MCP) server exposing framework knowledge and project operations directly to AI agents.

### Code Generator

Tools for generating accounts, instructions, tests, and boilerplate from structured specifications.

## Vision

AQAMI aims to become the standard AI-native development framework for Solana, enabling a future where developers and AI agents collaborate seamlessly to build secure, maintainable, and scalable blockchain applications.
