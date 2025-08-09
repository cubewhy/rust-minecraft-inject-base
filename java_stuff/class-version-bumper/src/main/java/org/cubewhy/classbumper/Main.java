package org.cubewhy.classbumper;

import java.io.File;
import java.io.FileOutputStream;
import java.util.jar.JarFile;
import java.util.zip.ZipOutputStream;

public class Main {
    public static void main(String[] args) throws Exception {
        String originJarPath = args[0];
        String outputJarPath = args[1];
        File originJarFile = new File(originJarPath);
        File outputJarFile = new File(outputJarPath);

        try (ZipOutputStream out = new ZipOutputStream(new FileOutputStream(outputJarFile))) {
            try (JarFile originJar = new JarFile(originJarFile)) {
                ClassVersionBumper.bumpJar(originJar, out);
            }
        }
    }
}