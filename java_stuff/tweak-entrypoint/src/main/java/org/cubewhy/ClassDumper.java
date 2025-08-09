package org.cubewhy;

import java.io.*;
import java.util.*;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;

public class ClassDumper {
    private final File baseDir;
    private final String dumpId = String.valueOf(System.currentTimeMillis() / 1000);

    public ClassDumper(File baseDir) {
        this.baseDir = baseDir;
    }

    public List<String> scanClasspathForClassNames() throws IOException {
        List<String> classNames = new ArrayList<>();
        String classpath = System.getProperty("java.class.path");
        String pathSeparator = File.pathSeparator;

        for (String path : classpath.split(pathSeparator)) {
            File f = new File(path);
            if (f.isDirectory()) {
                scanDirectoryForClasses(f, f.getAbsolutePath().length() + 1, classNames);
            } else if (f.isFile() && path.endsWith(".jar")) {
                scanJarForClasses(f, classNames);
            }
        }

        return classNames;
    }

    private void scanDirectoryForClasses(File dir, int basePathLen, List<String> classNames) {
        File[] files = dir.listFiles();
        if (files == null) return;

        for (File f : files) {
            if (f.isDirectory()) {
                scanDirectoryForClasses(f, basePathLen, classNames);
            } else if (f.getName().endsWith(".class")) {
                String absPath = f.getAbsolutePath();
                String relativePath = absPath.substring(basePathLen, absPath.length() - ".class".length());
                String className = relativePath.replace(File.separatorChar, '.');
                classNames.add(className);
            }
        }
    }

    private void scanJarForClasses(File jarFile, List<String> classNames) throws IOException {
        try (JarFile jar = new JarFile(jarFile)) {
            Enumeration<JarEntry> entries = jar.entries();
            while (entries.hasMoreElements()) {
                JarEntry entry = entries.nextElement();
                String name = entry.getName();
                if (name.endsWith(".class") && !entry.isDirectory()) {
                    String className = name.substring(0, name.length() - ".class".length()).replace('/', '.');
                    classNames.add(className);
                }
            }
        }
    }

    public void dumpClasses(List<String> classNames) {
        for (String className : classNames) {
            if (className.startsWith("java.") || className.startsWith("jdk.")) {
                continue; // skip internal classes
            }
            dumpClassByName(className);
        }
    }

    private void dumpClassByName(String className) {
        String classPath = "/" + className.replace('.', '/') + ".class";
        try (InputStream stream = ClassDumper.class.getResourceAsStream(classPath)) {
            if (stream == null) {
                System.out.printf("[WARN] Class resource not found: %s\n", className);
                return;
            }
            byte[] bytes = stream.readAllBytes();

            File outFile = new File(baseDir, String.format("dump-%s/%s.class", dumpId, className.replace('.', '/')));
            outFile.getParentFile().mkdirs();

            try (FileOutputStream fos = new FileOutputStream(outFile)) {
                fos.write(bytes);
            }

            System.out.printf("[INFO] Dumped class: %s\n", className);
        } catch (IOException e) {
            System.out.printf("[ERROR] Failed to dump class %s: %s\n", className, e.getMessage());
        }
    }

    public void dumpAll() throws IOException {
        List<String> classes = scanClasspathForClassNames();
        System.out.println("Total classes found: " + classes.size());
        this.dumpClasses(classes);
    }

    public static void main(String[] args) throws IOException {
        File outDir = new File("dumped_classes");
        ClassDumper dumper = new ClassDumper(outDir);
        dumper.dumpAll();
    }
}
