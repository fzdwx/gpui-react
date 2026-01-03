import React from 'react';
import { createRoot } from '../src/renderer/index';
import { App } from './virtual-list-app';

const root = createRoot();
root.render(React.createElement(App));

console.log('Virtual List demo running...');
console.log('Expecting window with virtual list of 10,000 items');

setTimeout(() => {
  console.log('Done! The virtual list window should be visible.');
  process.exit(0);
}, 10000);

process.on('SIGINT', () => {
  console.log('\\nShutting down...');
  process.exit(0);
});
