import React, { useRef, useState, useEffect, useCallback } from 'react';

interface VirtualListProps<T> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
  renderItem: (item: T, index: number) => React.ReactNode;
  overscan?: number;
}

interface VirtualListProps<T> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
  renderItem: (item: T, index: number) => React.ReactNode;
  overscan?: number;
}

export function VirtualList<T>({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  overscan = 5,
}: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(containerHeight);
  const [isScrolling, setIsScrolling] = useState(false);

  const containerRef = useRef<number | null>(null);
  const scrollTimeoutRef = useRef<number | null>(null);

  const getVisibleRange = useCallback((): { start: number; end: number } => {
    const start = Math.max(0, Math.floor((scrollTop - overscan * itemHeight) / itemHeight));
    const end = Math.min(
      items.length,
      Math.ceil((scrollTop + viewportHeight + overscan * itemHeight) / itemHeight)
    );
    return { start, end };
  }, [scrollTop, viewportHeight, itemHeight, overscan, items.length]);

  const { start: visibleStart, end: visibleEnd } = getVisibleRange();

  const handleScroll = useCallback((e: Event) => {
    if (scrollTimeoutRef.current) {
      clearTimeout(scrollTimeoutRef.current);
    }

    setIsScrolling(true);
    const newScrollTop = (e.currentTarget as HTMLElement).scrollTop;
    setScrollTop(newScrollTop);

    scrollTimeoutRef.current = setTimeout(() => {
      setIsScrolling(false);
    }, 150);
  }, []);

  useEffect(() => {
    if (containerRef.current === null) {
      containerRef.current = Math.floor(Math.random() * 100000);
    }
  }, []);

  const visibleItems = items.slice(visibleStart, visibleEnd);

  return React.createElement(
    'div',
    {
      style: {
        height: `${containerHeight}px`,
        overflow: 'auto',
        position: 'relative',
      },
      onScroll: handleScroll,
    },
    React.createElement(
      'div',
      {
        style: {
          height: `${items.length * itemHeight}px`,
          position: 'absolute',
          top: '0px',
          width: '100%',
        },
      },
      visibleItems.map((item, index) =>
        React.createElement(
          'div',
          {
            style: {
              position: 'absolute',
              top: `${(visibleStart + index) * itemHeight}px`,
              height: `${itemHeight}px`,
              width: '100%',
            },
            key: index,
          },
          renderItem(item, visibleStart + index)
        )
      )
    )
  );
}

export function VirtualList<T>({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  overscan = 5,
}: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(containerHeight);
  const [isScrolling, setIsScrolling] = useState(false);

  const containerRef = useRef<number | null>(null);
  const scrollTimeoutRef = useRef<number | null>(null);

  // Calculate visible range
  const getVisibleRange = useCallback((): { start: number; end: number } => {
    const start = Math.max(0, Math.floor((scrollTop - overscan * itemHeight) / itemHeight));
    const end = Math.min(
      items.length,
      Math.ceil((scrollTop + viewportHeight + overscan * itemHeight) / itemHeight)
    );
    return { start, end };
  }, [scrollTop, viewportHeight, itemHeight, overscan, items.length]);

  const { start: visibleStart, end: visibleEnd } = getVisibleRange();

  // Handle scroll events
  const handleScroll = useCallback((e: Event) => {
    if (scrollTimeoutRef.current) {
      clearTimeout(scrollTimeoutRef.current);
    }

    setIsScrolling(true);
    const newScrollTop = (e.currentTarget as HTMLElement).scrollTop;
    setScrollTop(newScrollTop);

    scrollTimeoutRef.current = setTimeout(() => {
      setIsScrolling(false);
    }, 150);
  }, []);

  // Register element with container
  useEffect(() => {
    if (containerRef.current === null) {
      containerRef.current = Math.floor(Math.random() * 100000);
    }
  }, []);

  const visibleItems = items.slice(visibleStart, visibleEnd);

  return React.createElement(
    'div',
    {
      style: {
        height: `${containerHeight}px`,
        overflow: 'auto',
        position: 'relative',
      },
    },
    React.createElement(
      'div',
      {
        style: {
          height: `${items.length * itemHeight}px`,
          position: 'absolute',
          top: '0px',
          width: '100%',
        },
      },
      visibleItems.map((item, index) =>
        React.createElement(
          'div',
          {
            style: {
              position: 'absolute',
              top: `${(visibleStart + index) * itemHeight}px`,
              height: `${itemHeight}px`,
              width: '100%',
            },
            key: index,
          },
          renderItem(item, visibleStart + index)
        )
      )
    )
  );
}
