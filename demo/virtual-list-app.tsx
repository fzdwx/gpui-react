import React, { useState, useEffect } from 'react';
import { createRoot } from '../src/renderer/index';
import { VirtualList } from '../src/components/VirtualList';

const LARGE_DATASET_SIZE = 10000;
const ITEM_HEIGHT = 50;

function App() {
  const [items, setItems] = useState<number[]>([]);

  useEffect(() => {
    const generateItems = () => {
      const newItems = Array.from({ length: LARGE_DATASET_SIZE }, (_, i) => i + 1);
      setItems(newItems);
    };

    generateItems();
  }, []);

  return React.createElement(
    'div',
    {
      style: {
        height: '600px',
        width: '800px',
        backgroundColor: '#1e1e1e',
        padding: '20px',
      },
    },
    React.createElement(
      'div',
      {
        style: {
          marginBottom: '20px',
          fontSize: '24px',
          fontWeight: 'bold',
          color: '#ffffff',
        },
      },
      `Virtual List Demo: ${LARGE_DATASET_SIZE.toLocaleString()} items`,
    ),
    React.createElement(
      'div',
      {
        style: {
          marginBottom: '10px',
          fontSize: '16px',
          color: '#888888',
        },
      },
      `Scroll to test virtualization - only visible items are rendered`,
    ),
    React.createElement(
      'div',
      {
        style: {
          height: '400px',
          width: '800px',
          border: '1px solid #333333',
          borderRadius: '8px',
        },
      },
      React.createElement(VirtualList, {
        items,
        itemHeight: ITEM_HEIGHT,
        containerHeight: 400,
        renderItem: (item, index) =>
          React.createElement(
            'div',
            {
              style: {
                height: `${ITEM_HEIGHT}px`,
                padding: '10px 20px',
                borderBottom: '1px solid #333333',
                backgroundColor: index % 2 === 0 ? '#2d2d2d' : '#1e1e1e',
              },
            },
            React.createElement('span', {
              style: { color: '#ffffff', fontSize: '14px' }
            }, `Item ${item}`)
          ),
        overscan: 10,
      }),
    )
  );
}
