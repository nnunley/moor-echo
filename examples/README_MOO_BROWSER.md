# Enhanced MOO Database Browser

A full-featured terminal-based interactive browser for exploring LambdaMOO database dumps with complete parsing and tabbed interface.

## Features

- **Multi-database support**: Browse Minimal.db, LambdaCore, ToastCore, and JaysHouseCore
- **Full database parsing**: Extracts actual property values and verb code from MOO databases
- **Modal database selector**: Press 'd' to switch between databases easily
- **Tabbed interface**: Four tabs for comprehensive object exploration:
  - **Overview**: Basic object information and statistics
  - **Properties**: Property names, values, owners, and permissions
  - **Verbs**: Verb names with actual MOO code display
  - **Relationships**: Parent-child hierarchy and location information
- **Player identification**: Players are highlighted in green with [P] markers
- **Scrollable details**: Full object information with scrolling support
- **Real data parsing**: Shows actual MOO code and property values, not placeholder text

## Usage

### Running the Browser

```bash
cargo run --example moo_db_browser
```

**Note**: This requires a proper terminal environment. It won't work in IDE terminals or certain remote shells.

### Controls

#### Main Object List View
- `↑/↓` - Navigate through objects
- `Enter` - View detailed information about selected object
- `d` - Open modal database selector
- `h` or `F1` - Show help screen
- `q` - Quit

#### Object Detail View
- `↑/↓` - Scroll through details in current tab
- `Tab` - Switch to next tab (Overview → Properties → Verbs → Relationships)
- `Shift+Tab` - Switch to previous tab
- `d` - Open modal database selector
- `Esc` - Return to object list
- `h` or `F1` - Show help screen
- `q` - Quit

#### Database Selector Modal
- `↑/↓` - Navigate through available databases
- `Enter` - Select database and return to object list
- `Esc` - Close selector without changing database

#### Help Screen
- `Esc` or `q` - Close help and return to previous view

### Interface Layout

#### Object List View
```
┌─ Database Header (shows current DB info) ─┐
│ LambdaCore (v4) - 97 objects, 1727 verbs  │
├────────────────────────────────────────────┤  
│ ► #0   The System Object                   │
│   #1   Root Class                          │
│   #2   Wizard [P] (parent: #1)            │
│   #3   generic room (parent: #1)          │
│   ...                                      │
└────────────────────────────────────────────┘
```

#### Object Detail View
```
┌─ Objects ─┐┌─ Object Details ─────────────┐
│ ► #0  Sys ││ Object #0: The System Object │
│   #1  Root││                              │
│   #2  Wiz ││ Basic Information:           │
│   ...     ││ ID: #0                       │
└───────────┘│ Name: The System Object     │
             │ Parent: None                 │
             │ Type: Object                 │
             │                              │
             │ Properties:                  │
             │   name                       │
             │   description                │
             │   location                   │
             └──────────────────────────────┘
```

### Color Coding

- **Yellow**: Core system objects (#0-#9)
- **Green**: Player objects with [P] marker
- **White**: Regular objects
- **Cyan**: Headers and titles
- **Highlighted**: Current selection with reversed colors

### Database Overview

The browser can display information from these MOO databases:

1. **Minimal.db** (4 objects)
   - Bare minimum MOO environment
   - System Object, Root Class, First Room, Wizard

2. **LambdaCore-latest.db** (97 objects)
   - Official LambdaMOO core database
   - Full set of generic classes and utilities
   - 1,727 verbs, 5 players

3. **ToastCore.db** (127+ objects)
   - Enhanced version of LambdaCore (Format Version 17)
   - Updated for ToastStunt server features
   - Additional objects and improvements
   - Note: Browser loads all actual objects found, regardless of header declaration

4. **JaysHouseCore.db** (238 objects)
   - Most comprehensive MOO core
   - 2,729 verbs, 8 players
   - Rich set of features and utilities

### Troubleshooting

**"Device not configured" error**: 
- Run in a proper terminal (Terminal.app on macOS, xterm, etc.)
- Don't run in IDE integrated terminals or tmux/screen sessions
- Make sure you have a TTY available

**Database not found**:
- Ensure database files are in the `examples/` directory
- Check that files were downloaded properly
- Verify file permissions

**Navigation issues**:
- Use arrow keys, not WASD
- Press `h` or `F1` for help if stuck
- Use `Esc` to go back to previous view

### Technical Details

The browser parses MOO database textdump formats (versions 1 and 4) and extracts:
- Object hierarchies and relationships
- Basic property information  
- Verb listings
- Player identification
- Object counts and statistics

This provides a read-only view of the database structure without needing a running MOO server.