const noop = async () => {};
const win = {
  isMaximized: async () => false,
  onResized: async () => () => {},
  onMoved: async () => () => {},
  minimize: noop,
  toggleMaximize: noop,
  close: noop,
  startResizeDragging: noop,
  scaleFactor: async () => 1,
  outerPosition: async () => ({ toLogical: () => ({ x: 0, y: 0 }) }),
  innerSize: async () => ({ toLogical: () => ({ width: 320, height: 320 }) }),
};
export const getCurrentWindow = () => win;
