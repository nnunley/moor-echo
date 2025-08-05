// XMLUI Chat System - UI generated from Echo database
// This creates a chat system where the UI is stored as object properties

// First, create a base XMLUI object
@create $xmlui named "XMLUI Base Object"

// Store XMLUI templates as properties
@property $xmlui.chat_window_template {"window": {"id": "chat-window", "title": "Echo Chat", "width": 800, "height": 600, "children": [{"scroll-view": {"id": "chat-messages", "height": 400, "children": [{"vertical-layout": {"id": "message-container", "padding": 10}}]}}, {"horizontal-layout": {"padding": 10, "children": [{"text-input": {"id": "chat-input", "placeholder": "Type your message...", "flex": 1}}, {"button": {"id": "send-button", "text": "Send", "on-click": "send_message"}}]}}]}}

@property $xmlui.message_templates {"system": {"horizontal-layout": {"padding": 5, "children": [{"label": {"text": "🔔", "color": "#4ec9b0"}}, {"label": {"text": "{message}", "color": "#4ec9b0", "margin-left": 5}}]}}, "player": {"horizontal-layout": {"padding": 5, "children": [{"label": {"text": "{player}:", "color": "#9cdcfe", "font-weight": "bold"}}, {"label": {"text": "{message}", "margin-left": 5}}]}}, "room": {"horizontal-layout": {"padding": 5, "children": [{"label": {"text": "[{room}]", "color": "#dcdcaa", "font-style": "italic"}}, {"label": {"text": "{message}", "margin-left": 5}}]}}}

// Create the chat room with XMLUI support
@create $xmlui_chat_room named "XMLUI Chat Room"
@parent $xmlui_chat_room $room

// Properties for the chat room
@property $xmlui_chat_room.ui_template $xmlui
@property $xmlui_chat_room.active_sessions {}

// Verb to render XMLUI from database objects
@verb $xmlui_chat_room:render_ui this none this
@program $xmlui_chat_room:render_ui
  // Get the UI template from the database
  template = this.ui_template.chat_window_template;
  
  // Convert to XMLUI format
  xmlui = this:object_to_xmlui(template);
  
  // Send to player
  player:tell_xmlui(xmlui);
.

// Helper verb to convert object structure to XMLUI
@verb $xmlui_chat_room:object_to_xmlui this none this
@program $xmlui_chat_room:object_to_xmlui
  obj = args[1];
  
  if (typeof(obj) == MAP)
    // Process each key-value pair
    for key in (keys(obj))
      value = obj[key];
      
      if (key in {"window", "scroll-view", "vertical-layout", "horizontal-layout", "text-input", "button", "label"})
        // This is an XMLUI element
        attrs = "";
        children = "";
        
        if (typeof(value) == MAP)
          for attr in (keys(value))
            if (attr == "children")
              // Process children recursively
              for child in (value[attr])
                children = children + this:object_to_xmlui(child);
              endfor
            else
              // Regular attribute
              attrs = attrs + " " + attr + "=\"" + tostr(value[attr]) + "\"";
            endif
          endfor
        endif
        
        if (children)
          return "<" + key + attrs + ">" + children + "</" + key + ">";
        else
          return "<" + key + attrs + "/>";
        endif
      endif
    endfor
  elseif (typeof(obj) == STR)
    return obj;
  else
    return tostr(obj);
  endif
.

// Enhanced notify verb that uses XMLUI
@verb $xmlui_chat_room:notify this none this
@program $xmlui_chat_room:notify
  {who, message} = args;
  
  // Determine message type and format
  if (who == #0)
    msg_type = "system";
    msg_data = {"message" -> message};
  elseif (valid(who) && is_player(who))
    msg_type = "player";
    msg_data = {"player" -> who.name, "message" -> message};
  else
    msg_type = "room";
    msg_data = {"room" -> this.name, "message" -> message};
  endif
  
  // Get the message template
  template = this.ui_template.message_templates[msg_type];
  
  // Apply the message data to the template
  rendered = this:apply_template(template, msg_data);
  
  // Send XMLUI update to all players in room
  for p in (this.contents)
    if (is_player(p) && p in keys(this.active_sessions))
      // Send as XMLUI append command
      p:tell_xmlui_update("message-container", "append", rendered);
    endif
  endfor
  
  // Also store in message history
  pass(@args);
.

// Template application verb
@verb $xmlui_chat_room:apply_template this none this
@program $xmlui_chat_room:apply_template
  {template, data} = args;
  result = template;
  
  // Simple template variable replacement
  for key in (keys(data))
    placeholder = "{" + key + "}";
    result = strsub(result, placeholder, tostr(data[key]));
  endfor
  
  return result;
.

// Verb called when player enters the room
@verb $xmlui_chat_room:accept this none this
@program $xmlui_chat_room:accept
  pass(@args);
  
  // Initialize XMLUI chat interface for the player
  fork (1)
    player:tell("Initializing chat interface...");
    this:render_ui();
    this.active_sessions[player] = time();
    
    // Load recent messages
    if (this.messages)
      for msg in (this.messages[$-10..$])
        {timestamp, who, text} = msg;
        this:notify(who, text);
      endfor
    endif
  endfork
.

// Verb to handle chat input from XMLUI
@verb $xmlui_chat_room:send_message this none this
@program $xmlui_chat_room:send_message
  // This would be called by the XMLUI button click
  message = args[1];
  
  if (message)
    this:notify(player, message);
  endif
.

// Create builtin functions for XMLUI
@verb $xmlui_chat_room:setup_xmlui_builtins this none this
@program $xmlui_chat_room:setup_xmlui_builtins
  // These would be implemented as actual builtins in Echo
  
  // tell_xmlui(player, xmlui_string) - Send XMLUI to player
  // tell_xmlui_update(player, element_id, action, content) - Update XMLUI element
  // xmlui_to_json(xmlui_object) - Convert XMLUI object to JSON
  // json_to_xmlui(json_string) - Convert JSON to XMLUI object
  
  player:tell("XMLUI builtins would be registered here.");
.