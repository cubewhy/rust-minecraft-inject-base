package org.cubewhy.classbumper;

import org.objectweb.asm.*;
import org.objectweb.asm.tree.*;

import java.io.IOException;
import java.io.InputStream;
import java.util.Enumeration;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.zip.ZipOutputStream;

public class ClassVersionBumper {
    public static byte[] bumpTo(byte[] originClassBytes, int targetVersion) {
        ClassReader cr = new ClassReader(originClassBytes);
        ClassNode cn = new ClassNode();
        cr.accept(cn, 8);
        if (cn.version >= 61) {
            return originClassBytes;
        }
        cn.version = targetVersion;
        ClassWriter cw = new ClassWriter(2);
        cn.accept(cw);
        return cw.toByteArray();
    }

    public static void bumpJar(JarFile originJar, ZipOutputStream outputJar) throws IOException {
        Enumeration<JarEntry> entries = originJar.entries();
        while (entries.hasMoreElements()) {
            JarEntry entry = entries.nextElement();
            InputStream is = originJar.getInputStream(entry);
            System.out.println(entry.getName());
            if (!entry.isDirectory()) {
                outputJar.putNextEntry(entry);
                if (!entry.getName().endsWith(".class")) {
                    byte[] bytes = is.readAllBytes();
                    outputJar.write(bytes, 0, bytes.length);
                } else {
                    byte[] originBytes = is.readAllBytes();
                    try {
                        byte[] bytes2 = bumpTo(originBytes, 61);
                        outputJar.write(bytes2, 0, bytes2.length);
                    } catch (Exception e) {
                        outputJar.write(originBytes, 0, originBytes.length);
                    }
                }
                outputJar.closeEntry();
            }
        }
    }
}
