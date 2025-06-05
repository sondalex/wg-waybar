# wg-waybar

![disconnected](assets/disconnected.png)
![connected](assets/connected.png)

This Waybar module provides a simple interface to monitor and toggle a WireGuard VPN connection directly from your Waybar. The module uses the [defguard_wireguard_rs](https://github.com/DefGuard/wireguard-rs) 

## Installation

1. **Clone the Repository** (or create a new project):
   ```bash
   git clone https://github.com/sondalex/wg-waybar.git
   cd wg-waybar

2. **Install**

   ```bash
   cargo build --release 
   sudo cp target/release/wg-waybar /usr/local/bin/
   ```

Add the executable to sudoers:


```bash
# /etc/sudoers.d/wg-waybar
user ALL=(ALL) NOPASSWD: /usr/local/bin/wg-waybar
```


## Configuration


1. **Configure Waybar**:
   Edit your Waybar configuration file (e.g., `~/.config/waybar/config`) to include the module:

   ```json
   {
       "modules-left": ["custom/vpn"],
       "custom/vpn": {
       "format": "{icon} {}",
       "tooltip": false,
       "format-icons": ["  ", "  ", "  "],
       "exec": "sudo /usr/local/bin/wg-waybar --signal 9 /etc/wireguard/<conf file>.conf",
       "return-type": "json",
       "signal": 9,
       "on-click": "sudo /usr/local/bin/wg-waybar --signal 9 /etc/wireguard/<conf file>.conf toggle"
       }
   }
   ```

2. Download rose-pine colors 

  ```bash
  wget -P $HOME/.config/waybar https://raw.githubusercontent.com/rose-pine/waybar/refs/heads/main/rose-pine-dawn.css
  wget -P $HOME/.config/waybar https://raw.githubusercontent.com/rose-pine/waybar/refs/heads/main/rose-pine-moon.css
  wget -P $HOME/.config/waybar https://raw.githubusercontent.com/rose-pine/waybar/refs/heads/main/rose-pine.css 
  ```

3. **Style Waybar**:
   Update your Waybar CSS (e.g., `~/.config/waybar/style.css`) to style the VPN module:
   ```css
   @import("rose-pine.css") 
   /*
    @import("rose-pine-moon.css")
   */
   /*
    @import url("rose-pine-dawn.css")
   */

    /*
    ...
    */

    #custom-vpn.connected {
       color: @foam;  /* Rose Pine Foam Dawn */
    }
    #custom-vpn.disconnected {
       color: @love;  /*Rose Pine Love Dawn */
    }
    #custom-vpn.error {
       color: @gold;  /*Rose Pine Gold dawn */
    }
   ```

4. **Restart Waybar**:
   Reload Waybar to apply the changes:

   ```bash
   pkill waybar
   waybar &
   ```
