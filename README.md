# Monitor-tui

Ever wanted to change your monitor layout in your terminal using a GUI?
No? Well here's an app that will let you do it anyway.

Based on rust tui, you can see your monitors as they appear, and modify
the position, resolution, and refresh rate all from the comfort of your
terminal!

Save your layouts using autorandr, export to monitors.xml and more (coming
soon).

## Debug mode
Run with `-d` to enable debug mode. This enables a 3 monitor
layout for testing purposes

## Todo
- [x] Help window
- [x] Handle disconnected monitors
- [x] Modify refresh rate
- [x] Modify resolution
- [ ] Presets (horizontal, vertical, defaults, etc)
- [x] Undo
- [ ] Autorandr integration
- [ ] Monitors.xml integration
- [x] Add debug mode
- [x] Update TUI to ratatui (why did I choose TUI when it's not
      maintained.....)
- [ ] Add incremental position changes
