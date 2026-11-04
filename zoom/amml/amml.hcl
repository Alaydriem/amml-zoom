namespace   = "alaydriem"
name        = "zoom"
entry       = "alaydriem_zoom.dll"
api_version = 2
entries = [
  "ClientInstance.update",
  "ClientInstance.getMouseGrabbed",
  "LevelRendererPlayer.getFov",
  "LevelRendererPlayer.onWillChangeDimension",
  "LocalPlayer.applyTurnDelta",
  "MinecraftGame.onAppFocusLost",
  "ClientExitLevel.hook",
]

listing {
  title       = "Zoom"
  summary     = "Hold a key to zoom, with smooth easing and matching mouse sensitivity."
  description = <<-EOT
    A configurable hold to zoom.
  EOT
  authors     = ["Charles R. Portwood II"]
  license     = "BSD-3-Clause"
  source      = "https://github.com/alaydriem/amml-zoom"
  homepage    = "https://github.com/alaydriem/amml-zoom#readme"
  tags        = ["zoom", "camera", "client", "utility"]
  icon        = "icon.png"
}
