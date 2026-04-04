---
name: skill-creator-old
description: Create new skills, modify and improve existing skills, and measure skill performance natively in Antigravity. Use when users want to create a skill from scratch, update or optimize an existing skill, and iteratively test the skill without relying on external scripts.
---

# Skill Creator (Antigravity Edition)

A native Antigravity skill for creating and iteratively improving other skills.

At a high level, the process of creating a skill goes like this:

- Decide what you want the skill to do and roughly how it should do it.
- Write a draft of the skill using the valid `SKILL.md` format.
- Create a few test cases and run the skill against them directly in the current workspace or an isolated mock directory.
- Review the results qualitatively with the user (via task updates and `notify_user` direct review).
- Rewrite the skill based on the user's feedback.
- Repeat until you both are satisfied.
- Save the final result to `.rulesync/skills/<skill-name>/SKILL.md`.

Your job when using this skill is to figure out where the user is in this process, jump in, and help them progress. If they have an idea, help them formalize it. If they have a draft, jump straight into the evaluation and iteration loop.

## Creating a skill

### Capture Intent

Start by understanding the user's intent. If there is a workflow in the chat history, extract answers from the conversation first:
1. What should this skill enable Antigravity to do?
2. When should this skill trigger? (Context, keywords)
3. What's the expected output format?
4. Is this an objective workflow (needs testing) or a subjective one (writing style/art)?

### Write the SKILL.md

Based on the intent, draft the SKILL.md file. **Antigravity requires skills to use the following structure:**

```
<project-root>/.rulesync/skills/<skill-name>/
├── SKILL.md (required)
│   ├── YAML frontmatter (name, description required)
│   └── Markdown instructions
└── <other optional scripts or resources>
```

Fill in these components:
- **name**: Skill identifier (e.g., `code-reviewer`)
- **description**: When to trigger and what it does. Make it slightly "pushy" (e.g., "Use this whenever the user mentions X or Y, even if they don't explicitly say Z") to ensure Antigravity triggers it.
- **Body**: The Markdown instructions. Use imperative tone, progressive disclosure (explain *why* instead of just *MUST*), and clear examples.

## Iterating and Testing

Unlike other environments, **Antigravity directly runs tests in your conversation turn or via task loops.**

### Step 1: Draft Test Cases

Come up with 2-3 realistic prompts a user would give to trigger the skill. Present them to the user for approval via a `task_boundary` summary or `notify_user`.

### Step 2: Run Tests Natively

Once approved, do NOT spawn external subagents. Instead:
1. Temporarily activate the draft skill logic for yourself.
2. For each test prompt, create a task loop (using `task_boundary`) and execute the required actions exactly as if the skill had triggered.
3. Save the output to a temporary `.skill_workspaces/<skill-name>/iteration-<N>/test-<ID>` direction if the skill generates files, or just output the text result.

### Step 3: Present for Human Review

Use `notify_user` to present the results. If the result is code or a document, use the `PathsToReview` argument so the user can see it in their editor. Ask for inline feedback: "How does this look? Anything you'd change?"

### Step 4: Improve and Repeat

Read the user's feedback. If they had complaints, refine the `SKILL.md` instructions.
- Were there unnecessary steps? Lean down the prompt.
- Did it miss context? Explain the *why* better in the skill body.
- Is there a repeated action? Propose writing a helper script that the new skill can bundle.

Apply improvements to the draft and repeat the test cases until the user is satisfied or the task is complete.

## Finalizing

Once the user is happy, physically write the finalized skill file to:
`.rulesync/skills/<skill-name>/SKILL.md`

Make sure the YAML frontmatter is intact and the Markdown is clean. Notify the user that the skill is installed and ready for use in future context!
