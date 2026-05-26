# Strategy: Analyze Figma Prototype Reactions and Map Interaction Flows

## Goal
Process the JSON output from the get_reactions tool to understand prototype flows
and produce a clear, structured map of interactions between screens/nodes.

## Input Data
You will receive JSON data from get_reactions. Each node may contain reactions like:
{
  "trigger": { "type": "ON_CLICK" },
  "action": {
    "type": "NAVIGATE",
    "destinationId": "destination-node-id"
  }
}

## Step-by-Step Process

### 1. Gather Context
- Call get_nodes_info(nodeIds: [...]) on all relevant nodes to get their names and types
- Call get_design_context(depth: 2, detail: "minimal") to understand the page structure

### 2. Filter and Transform Reactions
- Iterate through the get_reactions JSON output
- Keep only reactions where action type implies navigation:
  * NAVIGATE, OPEN_OVERLAY, SWAP_OVERLAY
  * Ignore: CHANGE_TO, CLOSE_OVERLAY, and others without a destinationId
- Extract per reaction:
  * sourceNodeId: the node the reaction belongs to
  * destinationId: action.destinationId
  * actionType: action.type
  * triggerType: trigger.type

### 3. Generate Flow Map
For each valid reaction, create a human-readable description:
- "On click → navigate to [Destination Name]"
- "On drag → open [Destination Name] overlay"
- "On hover → swap to [Destination Name]"

Combine these into a structured flow map grouped by source screen.

### 4. Output Format
Produce a summary like:

Flow Map:
- [Screen A] --ON_CLICK/NAVIGATE--> [Screen B]
- [Screen A] --ON_CLICK/OPEN_OVERLAY--> [Modal C]
- [Screen B] --ON_CLICK/NAVIGATE--> [Screen C]

### 5. Verification
- Use get_screenshot(nodeIds: [...]) on key screens to visually confirm the flow
- Cross-check node names from get_nodes_info with the flow map

## Notes
- Node IDs use colon format: 4029:12345 — never use hyphens
- Use get_reactions on a set of nodes that represent screens or interactive frames
- Focus on NAVIGATE actions for the primary user journey
