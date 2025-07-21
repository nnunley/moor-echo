#!/usr/bin/env node

const WebSocket = require('ws');

console.log('Testing UI functions through WebSocket...\n');

const ws = new WebSocket('ws://127.0.0.1:8081/ws');

ws.on('open', function open() {
    console.log('✓ Connected to WebSocket');
    
    // Test ui_clear()
    console.log('\n1. Testing ui_clear()...');
    ws.send(JSON.stringify({ 
        type: 'execute',
        command: 'ui_clear()'
    }));
});

ws.on('message', function message(data) {
    try {
        const event = JSON.parse(data);
        console.log('📨 Received:', JSON.stringify(event, null, 2));
        
        // After getting response for ui_clear, test ui_add_button
        if (event.type === 'result' && event.data && event.data.result === 'null') {
            setTimeout(() => {
                console.log('\n2. Testing ui_add_button()...');
                ws.send(JSON.stringify({ 
                    type: 'execute',
                    command: 'ui_add_button("test_btn", "Click Me", "print(\\"Button clicked!\\")")'
                }));
            }, 1000);
        }
        
        // After button test, test ui_add_text
        if (event.type === 'ui_update' && event.data && event.data.action === 'add_button') {
            setTimeout(() => {
                console.log('\n3. Testing ui_add_text()...');
                ws.send(JSON.stringify({ 
                    type: 'execute',
                    command: 'ui_add_text("test_text", "Hello from Echo!", {"color": "blue", "fontSize": "16px"})'
                }));
            }, 1000);
        }
        
        // After text test, test ui_update
        if (event.type === 'ui_update' && event.data && event.data.action === 'add_text') {
            setTimeout(() => {
                console.log('\n4. Testing ui_update()...');
                ws.send(JSON.stringify({ 
                    type: 'execute',
                    command: 'ui_update("test_text", {"text": "Updated text!", "color": "red"})'
                }));
            }, 1000);
        }
        
        // End test after ui_update
        if (event.type === 'ui_update' && event.data && event.data.action === 'update') {
            setTimeout(() => {
                console.log('\n✅ All tests completed! UI functions are working.');
                ws.close();
                process.exit(0);
            }, 1000);
        }
        
    } catch (e) {
        console.log('📨 Raw message:', data.toString());
    }
});

ws.on('close', function close() {
    console.log('🔌 WebSocket connection closed');
});

ws.on('error', function error(err) {
    console.error('❌ WebSocket error:', err);
    process.exit(1);
});

// Timeout after 15 seconds
setTimeout(() => {
    console.log('\n⏱️ Test timeout reached');
    ws.close();
    process.exit(1);
}, 15000);