---
title: "Deepnote: Cloud-Native Collaborative Data Science"
description: "Explore Deepnote's collaborative features, real-time editing, and cloud-native data science workflow compared to traditional notebooks."
author: "Mars Team"
date: "2026-11-14"
order: 10
tags: ["notebooks", "deepnote", "collaboration", "data-science", "cloud", "mars"]
---

# Why Deepnote? Collaborative Data Science Notebooks

Deepnote is a cloud-native data science platform that emphasizes real-time collaboration, team workflows, and integrated data infrastructure. Unlike traditional notebook environments, Deepnote is designed from the ground up for team collaboration and production-ready data science.

## What Deepnote Offers

### Real-Time Collaboration
- **Live editing**: Multiple users can edit notebooks simultaneously
- **Comments and mentions**: Built-in commenting and @mentions for team communication
- **Version history**: Track changes and revert to previous versions
- **Permissions**: Granular access control for team members

### Integrated Data Infrastructure
- **Data integrations**: Native connections to databases, data warehouses, and cloud storage
- **SQL support**: Write and execute SQL queries directly in notebooks
- **Data catalog**: Centralized data discovery and documentation
- **Scheduled runs**: Automate notebook execution on a schedule

### Cloud-Native Architecture
- **No setup required**: Runs entirely in the cloud, no local installation
- **Scalable compute**: Automatic resource scaling based on workload
- **GPU support**: Access to GPU-accelerated computing
- **Pre-installed libraries**: Common data science packages pre-installed

### Team Features
- **Shared workspaces**: Organize notebooks by project or team
- **Templates**: Reusable notebook templates for common workflows
- **Integrations**: Connect with Git, Slack, Jira, and other tools
- **Audit logs**: Track notebook access and changes

## Deepnote vs Traditional Notebooks

| Feature | Deepnote | Jupyter | Mars |
|---------|----------|---------|------|
| Collaboration | Real-time, built-in | Limited (via JupyterLab) | Planned |
| Cloud-native | Yes | No (local) | Desktop app |
| Data integrations | Native | Manual setup | Via runtime |
| Version control | Built-in | Git-based | Git-based |
| Scheduling | Built-in | Manual | CLI automation |
| Offline use | No | Yes | Yes (desktop) |

## When to Choose Deepnote

- **Team collaboration** is a primary requirement
- **Cloud-first workflow** with no local setup
- **Integrated data infrastructure** needed
- **Production data science** with scheduled runs
- **Enterprise features** like audit logs and permissions

## When Mars Might Be Better

- **Rust-first development** with native performance
- **Multi-language notebooks** (Rust, Python, R, Julia, C++)
- **Offline capability** with desktop application
- **CLI automation** for reproducible workflows
- **Open-source** and self-hosted options

## Mars's Approach to Collaboration

While Mars doesn't yet have Deepnote's real-time collaboration features, it provides:

- **Version control**: `.mars` files work with Git for version tracking
- **CLI automation**: Run notebooks headlessly for reproducible workflows
- **MCP integration**: AI assistant integration for enhanced productivity
- **Desktop application**: Native performance and offline capability
- **Multi-language support**: Mix languages in a single notebook

## The Future of Collaborative Notebooks

Both Deepnote and Mars represent different approaches to the notebook paradigm:

- **Deepnote**: Cloud-first, collaboration-focused, enterprise-ready
- **Mars**: Rust-first, multi-language, open-source, developer-focused

As data science teams grow, collaboration features become increasingly important. Mars is exploring ways to add collaboration while maintaining its Rust-first, multi-language advantages.

## Get Started

- [Mars Desktop App](packages/desktop) - Download the native application
- [Mars CLI](docs/12-cli.md) - Command-line interface for automation
- [Mars MCP](docs/18-mcp.md) - AI assistant integration
- [Getting Started](docs/01-getting-started.md) - First notebook tutorial
