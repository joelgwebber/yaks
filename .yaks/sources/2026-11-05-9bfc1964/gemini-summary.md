# Gemini's summary of "Full Course: Spec-Driven Development with Coding Agents"

Pasted by Joel Webber into a Delta thread on 2026-11-05. Written by Google Gemini from the video (not by Joel, not by the course). Timestamps in parentheses are Gemini's. A model's reading of the video, not a transcript: check a claim against the video before relying on it.

> This video presents **Spec-Driven Development (SDD)** as a disciplined, professional alternative to "vibe coding" (quick, iterative prompting) for building complex applications with AI coding agents. The core philosophy is to treat the AI agent as a highly capable **pair programmer** that provides speed, while you serve as the **senior architect** who provides clear, structured blueprints.
>
> ### Core Benefits of SDD
>
> *   **Control over large changes:** Small modifications to a spec can trigger large, consistent downstream changes in code (0:45).
> *   **Elimination of context decay:** Specs act as an external, persistent source of truth, preventing the loss of vital project context as agent sessions evolve (1:06).
> *   **High intent fidelity:** By forcing developers to define problems, constraints, and success criteria upfront, agents are significantly more likely to produce code that aligns with original goals (1:22).
>
> ### The SDD Workflow
>
> The workflow is divided into project-level foundation and feature-level iteration:
>
> 1.  **The Project Constitution (0:55, 15:00):** A formal document defining the project's **mission** (vision and audience), **tech stack** (tools and constraints), and **roadmap** (sequenced project phases). It serves as an agent-agnostic contract.
> 2.  **Feature Development Loops (2:39, 11:00):** A repeatable, branch-isolated process consisting of three steps:
>     *   **Plan:** Discuss the feature with the agent to create a spec covering tasks and validation requirements (20:12).
>     *   **Implement:** The agent builds the feature based on the approved spec (23:16).
>     *   **Validate:** The human-in-the-loop reviews the changes, runs tests, and ensures the code matches the intended architecture rather than just checking low-level details (25:20).
> 3.  **Replanning (11:08):** Between features, the developer steps back to update the constitution, refine the roadmap, and improve the workflow itself (32:10).
>
> ### Practical Implementation
>
> *   **Human-in-the-Loop:** Even with high-speed agents, the developer must remain the supervisor, reviewing diffs and preventing "AI fatigue" by keeping changes manageable and modular (26:18, 36:01).
> *   **Agent Skills:** Developers can create **skills** (custom instructions and tool wrappers) to automate repetitive parts of the SDD process, such as updating changelogs or running complex validation checks (33:23, 50:06).
> *   **Standardization:** The course emphasizes using protocols like **MCP (Model Context Protocol)** for external tools and **ACP (Agent Client Protocol)** to maintain flexibility, allowing developers to switch between agents and IDEs without losing their established workflow (52:13, 58:05).
> *   **Legacy Codebases:** SDD can be retrofitted onto existing projects by having the agent analyze current files to "reverse engineer" the constitution and roadmap before starting new feature development (46:01).
