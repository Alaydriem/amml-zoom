namespace    = "alaydriem"
name         = "zoom"
display_name = "Zoom"
version      = "26.60.6"
author       = "Alaydriem"
config_path  = "config.hcl"

claim "Camera" {
  mode   = "Shared"
  reason = "Changes the field of view and look speed while the zoom key is held"
}

keybind "zoom" {
  label       = "Zoom (hold)"
  default     = "Grave"
  capability  = "Camera"
  scope       = "Global"
  config_path = "keys.zoom"
}

section "zoom" {
  title = "Zoom"

  field "zoom.zoom_factor" {
    label   = "Zoom factor"
    help    = "The field of view is divided by this when fully zoomed"
    kind    = { float = { min = 1.0, max = 32.0, step = 0.5 } }
    default = 4.0
    apply   = "Live"
  }

  field "zoom.zoom_in_ms" {
    label   = "Zoom in time"
    help    = "Milliseconds to ease from the normal view to full zoom"
    kind    = { int = { min = 0, max = 2000, step = 10 } }
    default = 250
    apply   = "Live"
  }

  field "zoom.zoom_out_ms" {
    label   = "Zoom out time"
    help    = "Milliseconds to ease from full zoom back to the normal view"
    kind    = { int = { min = 0, max = 2000, step = 10 } }
    default = 250
    apply   = "Live"
  }

  field "zoom.easing" {
    label   = "Easing"
    help    = "The shape of the zoom movement"
    kind    = { enum = { options = [{ value = "linear", label = "Linear" }, { value = "easeOutSine", label = "Ease out (sine)" }, { value = "easeOutCubic", label = "Ease out (cubic)" }, { value = "easeOutQuint", label = "Ease out (quint)" }, { value = "easeOutExpo", label = "Ease out (expo)" }, { value = "easeInOutCubic", label = "Ease in and out (cubic)" }] } }
    default = "easeInOutCubic"
    apply   = "Live"
  }

  field "zoom.sensitivity_match_strength" {
    label   = "Look speed match"
    help    = "1 slows mouse look to exactly match the zoom; 0 leaves it unchanged"
    kind    = { float = { min = 0.0, max = 1.0, step = 0.05 } }
    default = 1.0
    apply   = "Live"
  }

  field "zoom.fade_variable_fov_while_zoomed" {
    label   = "Steady view while zoomed"
    help    = "Fades out the sprint and effect field of view changes while zoomed"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
}

section "keys" {
  title = "Keys"

  field "keys.zoom" {
    label   = "Zoom (hold)"
    help    = "Hold this key to zoom"
    kind    = "chord"
    default = "Grave"
    apply   = "Live"
  }
}
