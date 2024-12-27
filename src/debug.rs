pub mod xrandr_debug {
    // Mock data as a string to simulate command output
    pub const XRANDR_OUTPUT: &str = r#"Screen 0: minimum 320 x 200, current 5760 x 1440, maximum 16384 x 16384
HDMI-1 connected primary 2560x1440+0+0 (normal left inverted right x axis y axis) 597mm x 336mm
   2560x1440     60.00*+  59.95
   1920x1200     60.00
   1920x1080     60.00    59.94    50.00
   1600x1200     60.00
   1680x1050     60.00
   1280x1024     75.02    60.02
   1440x900      59.89
   1280x800      59.91
   1280x720      60.00    59.94    50.00
   1024x768      75.03    60.00
   800x600       75.00    60.32
   640x480       75.00    60.00    59.94
DP-1 connected 1920x1080+2560+0 (normal left inverted right x axis y axis) 521mm x 293mm
   1920x1080     60.00*+  59.94    50.00
   1680x1050     60.00
   1600x900      60.00
   1280x1024     75.02    60.02
   1440x900      59.89
   1280x800      59.91
   1280x720      60.00    59.94    50.00
   1024x768      75.03    60.00
   800x600       75.00    60.32
   640x480       75.00    60.00    59.94
DP-2 connected 1920x1080+4480+0 (normal left inverted right x axis y axis) 521mm x 293mm
   1920x1080     60.00*+  59.94    50.00
   1680x1050     60.00
   1600x900      60.00
   1280x1024     75.02    60.02
   1440x900      59.89
   1280x800      59.91
   1280x720      60.00    59.94    50.00
   1024x768      75.03    60.00
   800x600       75.00    60.32
   640x480       75.00    60.00    59.94"#;
}
