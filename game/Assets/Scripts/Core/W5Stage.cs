namespace Strobe.Core
{
    /// <summary>
    /// W5 Deepening stages (PATTERNS.md): how far a heroine has come
    /// in surfacing her own hidden desire / dissolving her own block.
    /// The stage is a state variable shared by progression, sessions,
    /// the phone hub, and the expression/feed views (ADAPT-SPEC A1-A5).
    /// </summary>
    public enum W5Stage
    {
        Unaware = 0,
        Curious = 1,
        Willing = 2,
        Craving = 3,
        Devoted = 4,
        SelfDriven = 5,
    }
}
