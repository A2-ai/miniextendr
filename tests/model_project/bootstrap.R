# bootstrap.R - Run before package build (Config/build/bootstrap: TRUE)
# Sets NOT_CRAN=true so configure runs in dev mode during devtools workflows.

env <- c(NOT_CRAN = "true")
env_strings <- paste0(names(env), "=", env)

if (.Platform$OS.type == "windows") {
  system2("bash", c("-l", "-c", "./configure.win"), env = env_strings)
} else {
  system2("./configure", env = env_strings)
}
