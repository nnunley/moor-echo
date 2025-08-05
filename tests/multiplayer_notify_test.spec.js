const { test, expect } = require('@playwright/test');

test.describe('Multiplayer notify() Demo', () => {
  test('two players can send notifications to each other', async ({ browser }) => {
    // Create two separate browser contexts (like two different users)
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();
    
    // Create pages for each player
    const player1 = await context1.newPage();
    const player2 = await context2.newPage();
    
    // Navigate both players to the Echo web interface
    await player1.goto('http://localhost:8081');
    await player2.goto('http://localhost:8081');
    
    // Wait for the web interface to load
    await player1.waitForTimeout(2000); // Give server time to start
    await player1.waitForSelector('#commandInput', { timeout: 15000 });
    await player2.waitForSelector('#commandInput', { timeout: 15000 });
    
    console.log('Both players connected to Echo web interface');
    
    // Since we're using Echo REPL, we need to create players differently
    // The web server should already have a runtime available
    
    // Wait for connection messages
    await player1.waitForTimeout(1000);
    await player2.waitForTimeout(1000);
    
    // For this demo, we'll use the existing root (#1) and system (#0) objects
    // since they are pre-created in the MOO ID map
    
    // Set up message monitoring for both players
    const alice_messages = [];
    const bob_messages = [];
    
    // Monitor WebSocket messages for player 1 (Alice)
    player1.on('websocket', ws => {
      ws.on('framereceived', event => {
        try {
          const data = JSON.parse(event.payload);
          console.log('[Player 1] WebSocket received:', JSON.stringify(data));
          if (data.type === 'ChatMessage' && data.data.player.includes('system@')) {
            alice_messages.push(data.data.message);
            console.log('Alice received notification:', data.data.message);
          }
        } catch (e) {
          console.log('[Player 1] Failed to parse:', e.message);
        }
      });
    });
    
    // Monitor WebSocket messages for player 2 (Bob)
    player2.on('websocket', ws => {
      ws.on('framereceived', event => {
        try {
          const data = JSON.parse(event.payload);
          console.log('[Player 2] WebSocket received:', JSON.stringify(data));
          if (data.type === 'ChatMessage' && data.data.player.includes('system@')) {
            bob_messages.push(data.data.message);
            console.log('Bob received notification:', data.data.message);
          }
        } catch (e) {
          console.log('[Player 2] Failed to parse:', e.message);
        }
      });
    });
    
    console.log('\n--- Testing MOO notify() between connections ---\n');
    
    // Connection 1 sends a message to root object
    await player1.fill('#commandInput', 'notify(1, "Hello from Connection 1!")');
    await player1.press('#commandInput', 'Enter');
    await player1.waitForTimeout(1000);
    
    // Connection 2 sends a message to root object
    await player2.fill('#commandInput', 'notify(1, "Hi from Connection 2!")');
    await player2.press('#commandInput', 'Enter');
    await player2.waitForTimeout(1000);
    
    // Test using notify with object 0 (system)
    await player1.fill('#commandInput', 'notify(0, "System notification test")');
    await player1.press('#commandInput', 'Enter');
    await player1.waitForTimeout(1000);
    
    // Test multiple notifications in sequence
    await player1.fill('#commandInput', 'notify(1, "Message 1 of 3")');
    await player1.press('#commandInput', 'Enter');
    await player1.waitForTimeout(500);
    
    await player1.fill('#commandInput', 'notify(1, "Message 2 of 3")');
    await player1.press('#commandInput', 'Enter');
    await player1.waitForTimeout(500);
    
    await player1.fill('#commandInput', 'notify(1, "Message 3 of 3")');
    await player1.press('#commandInput', 'Enter');
    await player1.waitForTimeout(1000);
    
    // Wait a bit for messages to appear
    await player1.waitForTimeout(2000);
    
    // Take screenshots of both player windows
    await player1.screenshot({ path: 'alice-chat.png', fullPage: true });
    await player2.screenshot({ path: 'bob-chat.png', fullPage: true });
    
    console.log('\n--- Screenshots saved ---');
    console.log('Player 1 screenshot: alice-chat.png');
    console.log('Player 2 screenshot: bob-chat.png');
    
    // Try to get output content
    try {
      const output1 = await player1.locator('#outputContainer').textContent();
      console.log('\n--- Player 1 Output ---');
      console.log(output1 || '(empty)');
    } catch (e) {
      console.log('Could not read player 1 output:', e.message);
    }
    
    try {
      const output2 = await player2.locator('#outputContainer').textContent();
      console.log('\n--- Player 2 Output ---');
      console.log(output2 || '(empty)');
    } catch (e) {
      console.log('Could not read player 2 output:', e.message);
    }
    
    // Clean up
    await context1.close();
    await context2.close();
  });
});