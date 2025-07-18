// test_injections.moo - Demonstrates language injection features

// SQL injection in strings
query = "SELECT * FROM players WHERE name = 'Alice'";
db_query("INSERT INTO logs (player, action) VALUES ($1, $2)", {player_id, action});

// Regular expression patterns
pattern = match(text, "^[A-Za-z0-9]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$");
result = obj:match("[0-9]{3}-[0-9]{3}-[0-9]{4}");  // Phone number pattern

// MOO code in eval contexts
eval("player.name = \"NewName\"; player:save();");
compile("return x * 2;");

// JSON data
config = parse_json("{\"theme\": \"dark\", \"fontSize\": 14, \"autoSave\": true}");
data = from_json("[1, 2, 3, {\"nested\": true}]");

// HTML in properties (object definition)
object WebPage
  property html (owner: #1) = "<html><body><h1>Welcome</h1></body></html>";
  property style (owner: #1) = "body { font-family: Arial; color: #333; }";
  property template (owner: #1) = "<div class='container'>{{content}}</div>";
endobject

/**
 * # Documentation Comment
 * 
 * This is a **markdown** formatted comment that could be
 * highlighted differently in editors that support it.
 * 
 * - Feature 1
 * - Feature 2
 * 
 * ```moo
 * // Example code
 * player:tell("Hello!");
 * ```
 */
fn documented_function()
  // Function implementation
  return true;
endfn