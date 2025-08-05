// MOO Chat Room Implementation with XMLUI

// Create a basic chat room object
@create $room named "Chat Room"

// Add properties for managing chat
@property $room.messages []
@property $room.max_messages 100
@property $room.ui_template #0

// Verb to handle incoming messages and format them
@verb $room:notify this none this
@program $room:notify
  {who, message} = args;
  player = who;
  
  // Store message in room history
  this.messages = {@this.messages, {time(), player, message}}[$ - this.max_messages + 1..$];
  
  // Format message based on type
  if (player == #0)
    // System message
    formatted = {"type" -> "system", "message" -> message};
  elseif (typeof(player) == OBJ)
    // Player message
    pname = valid(player) ? player.name | tostr(player);
    formatted = {"type" -> "player", "player" -> pname, "message" -> message};
  else
    // Room message
    formatted = {"type" -> "room", "room" -> this.name, "message" -> message};
  endif
  
  // Send to all players in the room
  for p in (this.contents)
    if (is_player(p))
      notify(p, tojson(formatted));
    endif
  endfor
.

// Verb to handle chat commands
@verb $room:say this none none
@program $room:say
  if (!args || !args[1])
    player:tell("Say what?");
    return;
  endif
  
  message = args[1];
  this:notify(player, message);
  
  // Also show it locally formatted
  player:tell("You say, \"" + message + "\"");
.

// Verb to show recent messages when entering
@verb $room:look_self this none this
@program $room:look_self
  pass(@args);
  
  if (this.messages)
    player:tell("");
    player:tell("Recent messages:");
    player:tell("-" * 40);
    
    // Show last 10 messages
    recent = this.messages[$-9..$];
    for msg in (recent)
      {timestamp, who, text} = msg;
      if (typeof(who) == OBJ && valid(who))
        player:tell(who.name + ": " + text);
      else
        player:tell("[System] " + text);
      endif
    endfor
  endif
.

// Verb to broadcast room announcements
@verb $room:announce this none this
@program $room:announce
  if (!wizard)
    return E_PERM;
  endif
  
  message = args[1];
  this:notify(#0, "[" + this.name + "] " + message);
.

// XMLUI integration verb
@verb $room:show_chat_ui this none this
@program $room:show_chat_ui
  // This would load and display the XMLUI chat interface
  // In a real implementation, this would use Echo's XMLUI system
  player:tell("Loading chat interface...");
  
  // Send XMLUI command to load the chat window
  notify(player, "@xmlui load /examples/xmlui_chat.xml");
  
  // Populate with recent messages
  for msg in (this.messages[$-20..$])
    {timestamp, who, text} = msg;
    msg_data = {"time" -> timestamp, "player" -> tostr(who), "message" -> text};
    notify(player, "@xmlui update chat-messages append " + tojson(msg_data));
  endfor
.