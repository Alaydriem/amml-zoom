namespace    = "alaydriem"
name         = "zoom"
entry        = "alaydriem_zoom.dll"
api_version  = 3

uses {
  feature "zoom" {
    event "client.tick" {
      version  = 1
      required = true
    }
    event "camera.fov" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "registry.provider_gone" {
      version  = 1
      required = true
    }
    area "client.mouse" {
      version  = 1
      required = true
    }
    event "mods.loaded" {
      version  = 1
      required = false
    }
    event "input.lease_suspended" {
      version  = 1
      required = false
    }
    event "input.lease_resumed" {
      version  = 1
      required = false
    }
    event "input.lease_lost" {
      version  = 1
      required = false
    }
    event "input.injected_released" {
      version  = 2
      required = false
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = false
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = false
    }
    area "input.lease" {
      version  = 1
      required = false
    }
  }
  feature "look" {
    event "input.look_delta" {
      version  = 1
      required = false
    }
  }
  feature "reset" {
    event "window.focus" {
      version  = 1
      required = false
    }
    event "world.left" {
      version  = 1
      required = false
    }
    event "world.dimension_changing" {
      version  = 1
      required = false
    }
  }
}

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
