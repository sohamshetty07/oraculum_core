import re
import random
import json

# --- MOCK DATA (Simulating what our Scout found) ---
# In production, this comes from the Reddit/Wiki APIs we just tested.
MOCK_VOICES = [
    "Review: Tastes amazing but too sweet.",
    "Is it really 25g protein? Feels heavy.",
    "Best budget option for students.",
    "Found mold in my pack, terrible QC!",
    "Better than Whey protein for the price.",
    "Rose flavor is artificial, stick to plain.",
    "Causes bloating if you are lactose intolerant.",
    "Price increased to ₹80, not worth it anymore.",
    "My gym trainer recommended this.",
    "Packaging is hard to open."
]

MOCK_FACTS = {
    "product_name": "Amul Protein Lassi",
    "nutriscore": "B",
    "protein": "15g"
}

# --- LOGIC 1: RECURSIVE QUERY RELAXATION ---
def clean_query_step(query):
    """
    Intelligently strips noise from a query to find the 'Core Identity'.
    Returns the next simpler version of the query.
    """
    original = query
    
    # Step 1: Remove specific measurements (200ml, 1kg, 500g)
    query = re.sub(r'\b\d+(ml|g|kg|L|oz)\b', '', query, flags=re.IGNORECASE)
    
    # Step 2: Remove "Pack of X" or parentheticals
    query = re.sub(r'\(.*?\)|pack of \d+', '', query, flags=re.IGNORECASE)
    
    # Step 3: Remove common stopwords/prepositions
    query = re.sub(r'\b(for|with|and|flavor|flavour|variant)\b', '', query, flags=re.IGNORECASE)
    
    # Step 4: Collapse whitespace
    query = " ".join(query.split())
    
    if query == original:
        # If regex didn't change anything, chop off the last word (brute force)
        tokens = query.split()
        if len(tokens) > 1:
            query = " ".join(tokens[:-1])
            
    return query

def robust_search_simulation(messy_input):
    print(f"\n🔍 ROBUSTNESS TEST: Input = '{messy_input}'")
    current_q = messy_input
    
    # Simulate API attempts
    for attempt in range(4):
        print(f"   Attempt {attempt+1}: Searching for '{current_q}'...")
        
        # SIMULATION: We pretend 'Amul Protein Lassi' is the only valid key in our DB
        if "amul protein lassi" in current_q.lower() and len(current_q) < 25:
            print("   ✅ SUCCESS: Match found in Database!")
            return True
            
        # If fail, relax the query
        new_q = clean_query_step(current_q)
        if new_q == current_q or len(new_q) < 3:
            print("   ❌ FAILED: Query cannot be simplified further.")
            return False
        current_q = new_q
        
    return False

# --- LOGIC 2: CONTEXT SHARDING (SCALABILITY) ---
def shard_context(agent_id, total_agents, all_voices, facts):
    """
    Slices the world's knowledge so different agents see different things.
    """
    # 1. Determine the 'Lens' based on Agent ID
    # Agents are split into clusters
    shard_size = max(1, len(all_voices) // 3)
    
    if agent_id % 3 == 0:
        # Cluster A: Optimists (First 1/3rd of comments)
        my_voices = all_voices[:shard_size]
        focus = "Positive Reviews"
    elif agent_id % 3 == 1:
        # Cluster B: Critics (Middle 1/3rd of comments)
        my_voices = all_voices[shard_size:shard_size*2]
        focus = "Critical Reviews"
    else:
        # Cluster C: Realists (Last 1/3rd)
        my_voices = all_voices[shard_size*2:]
        focus = "Recent Issues"

    return {
        "agent_id": agent_id,
        "focus_lens": focus,
        "knowledge_fragment": my_voices,
        "facts": facts # Everyone sees facts
    }

if __name__ == "__main__":
    # TEST 1: Handle a terrible user input
    user_input = "Amul Protein Lassi (Rose Flavour) 200ml Pack of 6 for Gym"
    robust_search_simulation(user_input)
    
    # TEST 2: Scale to 10 Agents with Sharding
    print(f"\n👥 SCALABILITY TEST: Generating unique contexts for 10 agents...")
    for i in range(1, 11):
        context = shard_context(i, 10, MOCK_VOICES, MOCK_FACTS)
        print(f"   Agent {i} sees [{context['focus_lens']}]: {len(context['knowledge_fragment'])} specific comments.")
        # Print sample for Agent 6 to prove they see different stuff than Agent 1
        if i == 6:
            print(f"      -> Agent 6 reads: '{context['knowledge_fragment'][0]}'")
        if i == 1:
            print(f"      -> Agent 1 reads: '{context['knowledge_fragment'][0]}'")