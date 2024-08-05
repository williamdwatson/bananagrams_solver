/**
 * A basic wrapper around a link
 * 
 * @component
 */
export default function LinkWrapper({href, children} : { href: string, children: string }) {
    return <a href={href} title={href} target="_blank">{children}</a>
}