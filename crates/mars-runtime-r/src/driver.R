# Mars R kernel driver.
#
# Speaks newline-delimited JSON on stdin/stdout. Cell sources are parsed and
# evaluated in .GlobalEnv so bindings persist across cells. Printed output and
# messages are captured per cell and forwarded as ``stream`` messages; the
# value of a trailing visible expression is forwarded as ``display`` data,
# and plots drawn on the graphics device are captured as image data.
#
# Requires the jsonlite package (Rscript -e 'install.packages("jsonlite")').
#
# Request messages (one JSON object per line on stdin):
#
# - ``{"type": "execute", "id": N, "source": "..."}``
# - ``{"type": "inspect", "id": N}``
# - ``{"type": "check", "id": N, "source": "..."}``
# - ``{"type": "shutdown"}``
#
# Response messages (one JSON object per line on stdout):
#
# - ``{"type": "stream", "id": N, "stream": "stdout"|"stderr", "data": "..."}``
# - ``{"type": "display", "id": N, "data": [[mime, data], ...]}``
# - ``{"type": "result", "id": N, "ok": bool, "error": {...} | null}``
# - ``{"type": "inspection", "id": N, "variables": [...]}``
# - ``{"type": "check_result", "id": N, "error": {...} | null}``
#
# Known limitations:
#
# - ``readline()``/``scan()`` inside cell code consume kernel protocol lines.
# - ``cat(..., file = stderr())`` bypasses capture and lands in the driver
#   process's stderr tail instead of stream output.
# - Cells that replace the graphics device themselves may lose plot capture.
# - Plot capture requires a working png or pdf device; pdf is the fallback.

local({
  `%||%` <- function(a, b) if (is.null(a)) b else a

  if (!requireNamespace("jsonlite", quietly = TRUE)) {
    cat(
      "mars-r-driver: the jsonlite package is required; ",
      "install it with install.packages(\"jsonlite\")\n",
      file = stderr()
    )
    quit(save = "no", status = 1)
  }

  current_id <- NULL

  send_msg <- function(msg) {
    if (is.null(msg$id) && !is.null(current_id)) {
      msg$id <- current_id
    }
    line <- jsonlite::toJSON(
      msg,
      auto_unbox = TRUE,
      null = "null",
      na = "null",
      digits = NA
    )
    cat(line, "\n", file = stdout())
    flush.console()
  }

  emit_stream <- function(stream, data) {
    if (!is.null(data) && nzchar(data)) {
      send_msg(list(type = "stream", stream = stream, data = data))
    }
  }

  error_payload <- function(condition, calls) {
    list(
      name = class(condition)[1],
      message = conditionMessage(condition),
      traceback = vapply(
        utils::head(calls, 20L),
        function(call) paste(deparse(call), collapse = " "),
        character(1)
      )
    )
  }

  # Evaluate one cell. Returns list(stdout, stderr, value, visible, error,
  # plot_file, plot_mime). stdout/stderr are character vectors of lines.
  eval_cell <- function(source) {
    stderr_lines <- character()
    value <- NULL
    visible <- FALSE
    cell_error <- NULL
    error_calls <- list()

    use_png <- isTRUE(capabilities("png")[["png"]])
    plot_file <- tempfile(fileext = ".png")
    device_open <- FALSE
    plot_mime <- NULL

    if (use_png) {
      device_open <- tryCatch(
        {
          grDevices::png(plot_file, width = 640, height = 480)
          TRUE
        },
        error = function(e) FALSE
      )
      if (device_open) {
        plot_mime <- "image/png"
      }
    }
    if (!device_open) {
      plot_file <- tempfile(fileext = ".pdf")
      device_open <- tryCatch(
        {
          grDevices::pdf(plot_file, width = 7, height = 5)
          TRUE
        },
        error = function(e) FALSE
      )
      if (device_open) {
        plot_mime <- "application/pdf"
      }
    }
    device_number <- if (device_open) grDevices::dev.cur() else NULL

    stdout_lines <- capture.output(
      withCallingHandlers(
        tryCatch(
          {
            exprs <- parse(text = source)
            count <- length(exprs)
            for (i in seq_len(count)) {
              if (i == count) {
                vis <- withVisible(eval(exprs[[i]], envir = .GlobalEnv))
                value <- vis$value
                visible <- vis$visible
              } else {
                eval(exprs[[i]], envir = .GlobalEnv)
              }
            }
          },
          error = function(e) {
            cell_error <<- e
            error_calls <<- sys.calls()
          }
        ),
        message = function(m) {
          stderr_lines <<- c(
            stderr_lines,
            paste0("message: ", conditionMessage(m))
          )
          invokeRestart("muffleMessage")
        },
        warning = function(w) {
          stderr_lines <<- c(
            stderr_lines,
            paste0("Warning: ", conditionMessage(w))
          )
          invokeRestart("muffleWarning")
        }
      ),
      type = "output"
    )

    # Detect whether anything was drawn before closing the device. Devices
    # without a display list make recordPlot() fail; in that case fall back
    # to a non-empty file as the signal.
    plotted <- FALSE
    if (is.null(cell_error) && device_open) {
      recorded <- tryCatch(
        grDevices::recordPlot(),
        error = function(e) NULL
      )
      if (is.null(recorded)) {
        plotted <- TRUE
      } else {
        plotted <- length(recorded) > 0
      }
    }

    if (device_open && !is.null(device_number)) {
      tryCatch(
        grDevices::dev.off(which = device_number),
        error = function(e) NULL
      )
      device_open <- FALSE
    }

    has_plot <- is.null(cell_error) &&
      plotted &&
      file.exists(plot_file) &&
      file.info(plot_file)$size > 0

    list(
      stdout = stdout_lines,
      stderr = stderr_lines,
      value = value,
      visible = visible,
      error = cell_error,
      error_calls = error_calls,
      plot_file = if (has_plot) plot_file else NULL,
      plot_mime = plot_mime
    )
  }

  display_value <- function(result) {
    if (isTRUE(result$visible)) {
      printed <- tryCatch(
        capture.output(print(result$value)),
        error = function(e) character()
      )
      parts <- list()
      if (length(printed) > 0) {
        parts <- c(
          parts,
          list(list("text/plain", paste(printed, collapse = "\n")))
        )
      }
      if (!is.null(result$plot_file)) {
        raw <- readBin(
          result$plot_file,
          what = "raw",
          n = file.info(result$plot_file)$size
        )
        parts <- c(
          parts,
          list(list(result$plot_mime, jsonlite::base64_enc(raw)))
        )
        unlink(result$plot_file)
      }
      if (length(parts) > 0) {
        send_msg(list(type = "display", data = parts))
      }
    } else if (!is.null(result$plot_file)) {
      raw <- readBin(
        result$plot_file,
        what = "raw",
        n = file.info(result$plot_file)$size
      )
      send_msg(list(
        type = "display",
        data = list(list(result$plot_mime, jsonlite::base64_enc(raw)))
      ))
      unlink(result$plot_file)
    }
  }

  handle_execute <- function(request) {
    id <- request$id
    source <- request$source %||% ""
    current_id <<- id

    result <- eval_cell(source)

    stdout_text <- paste(result$stdout, collapse = "\n")
    if (length(result$stdout) > 0) {
      stdout_text <- paste0(stdout_text, "\n")
    }
    emit_stream("stdout", stdout_text)

    stderr_text <- paste(result$stderr, collapse = "\n")
    if (length(result$stderr) > 0) {
      stderr_text <- paste0(stderr_text, "\n")
    }
    emit_stream("stderr", stderr_text)

    if (is.null(result$error)) {
      display_value(result)
    }

    payload <- NULL
    ok <- is.null(result$error)
    if (!ok) {
      payload <- error_payload(result$error, result$error_calls)
    }
    send_msg(list(type = "result", id = id, ok = ok, error = payload))
    current_id <<- NULL
  }

  handle_inspect <- function(request) {
    names <- ls(.GlobalEnv, all.names = TRUE)
    names <- names[!grepl("^\\.", names)]
    variables <- lapply(names, function(nm) {
      val <- get(nm, envir = .GlobalEnv, inherits = FALSE)
      type <- class(val)[1]
      preview <- tryCatch(
        {
          text <- paste(
            capture.output(utils::str(val, max.level = 1, give.attr = FALSE)),
            collapse = "\n"
          )
          if (nchar(text) > 160) {
            paste0(substr(text, 1, 157), "...")
          } else {
            text
          }
        },
        error = function(e) NULL
      )
      list(name = nm, type = type, value = preview)
    })
    send_msg(list(type = "inspection", id = request$id, variables = variables))
  }

  handle_check <- function(request) {
    err <- tryCatch(
      {
        parse(text = request$source %||% "")
        NULL
      },
      error = function(e) e
    )
    payload <- NULL
    if (!is.null(err)) {
      payload <- list(
        name = class(err)[1],
        message = conditionMessage(err),
        lineno = 1,
        offset = 1
      )
    }
    send_msg(list(type = "check_result", id = request$id, error = payload))
  }

  input_con <- file("stdin")
  repeat {
    line <- tryCatch(
      readLines(input_con, n = 1L, warn = FALSE),
      error = function(e) character()
    )
    if (length(line) == 0) {
      break
    }
    line <- trimws(line)
    if (!nzchar(line)) {
      next
    }
    request <- tryCatch(
      jsonlite::fromJSON(line, simplifyVector = FALSE),
      error = function(e) NULL
    )
    if (is.null(request) || is.null(request$type)) {
      next
    }
    done <- FALSE
    switch(
      request$type,
      execute = handle_execute(request),
      inspect = handle_inspect(request),
      check = handle_check(request),
      shutdown = {
        done <- TRUE
      },
      NULL
    )
    if (done) {
      break
    }
  }
})
