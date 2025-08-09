plugins {
    id("java")
    application
}

group = "org.cubewhy"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.ow2.asm:asm:9.8")
    implementation("org.ow2.asm:asm-tree:9.8")
}

application {
    mainClass = "org.cubewhy.classbumper.Main"
}
