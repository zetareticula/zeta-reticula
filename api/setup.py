from setuptools import setup
from pathlib import Path

setup(
    name="zeta_reticula_api",
    version="0.1.0",
    description="Python bindings for Zeta Reticula API",
    long_description=Path("README.md").read_text(),
    long_description_content_type="text/markdown",
    packages=["zeta_reticula_api"],
    package_dir={"": "."},
    rust_extensions=[RustExtension("zeta_reticula_api.zeta_reticula_api", "api/Cargo.toml")],
    zip_safe=False,
)