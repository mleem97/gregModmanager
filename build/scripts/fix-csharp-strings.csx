// Dev-Helper Script zur Behebung von C# Interpolation Issues
// ==============================================================
//
// Hintergrund: Raw string literals ($""") mit Interpolation erfordern doppelte geschweifte Klammern
// für die eigentliche C#-Interpolation.
//
// Verwendung: dotnet script fix-csharp-strings.csx

using System;
using System.IO;
using System.Text.RegularExpressions;

var path = "Services/WorkspaceService.cs";
if (!File.Exists(path)) {
    Console.WriteLine($"ERROR: File not found: {path}");
    return;
}

var code = File.ReadAllText(path);

// Raw string literals ($""") mit Interpolation müssen doppelte Klammern verwenden
// für die Interpolationsvariablen {projectName} und {dirName}
code = code.Replace("mainCs = $\"\"\"", "mainCs = 345\"\"\"");
code = code.Replace("{projectName}", "{{projectName}}");
code = code.Replace("{dirName}", "{{dirName}}");

File.WriteAllText(path, code);
Console.WriteLine("✓ C# string interpolation fixed in WorkspaceService.cs");
