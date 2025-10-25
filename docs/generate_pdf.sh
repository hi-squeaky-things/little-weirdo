echo "⚙️ Start building the PDF"
podman run -q -v ${PWD}:/documents docker.io/asciidoctor/docker-asciidoctor asciidoctor-pdf -r asciidoctor-diagram little_weirdo_synth_design.adoc
rm *.png
